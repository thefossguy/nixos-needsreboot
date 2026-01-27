{
  description = "Check if NixOS needs a reboot after system updates";

  # inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  # Always use the stable release on flakehub.
  inputs.nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/*.tar.gz";

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [
        "aarch64-linux"
        "riscv64-linux"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = self.packages.${system}.nixos-needsreboot;

          nixos-needsreboot = pkgs.writeShellApplication {
            name = "nixos-needsreboot";

            runtimeInputs = with pkgs; [ coreutils findutils ];

            text = ''
              VERSION="0.3.0"
              OLD_SYSTEM_PATH="/run/booted-system"
              NEW_SYSTEM_PATH="/nix/var/nix/profiles/system"
              NIXOS_NEEDS_REBOOT="/var/run/reboot-required"

              DRY_RUN=false
              for arg in "$@"; do
                case "$arg" in
                  --version)
                    echo "nixos-needsreboot: v''${VERSION}"
                    exit 0
                    ;;
                  --dry-run)
                    DRY_RUN=true
                    ;;
                esac
              done

              if [[ "$EUID" -ne 0 && "$DRY_RUN" == "false" ]]; then
                echo "ERROR: please run this as root"
                echo "HINT: use the '--dry-run' option"
                exit 1
              fi

              if [[ ! -e "$NEW_SYSTEM_PATH" ]]; then
                echo "This binary is intended to run only on NixOS." >&2
                exit 1
              fi

              get_kernel_version() {
                local kernel_path="$1"
                local modules_dir="''${kernel_path}/lib/modules"
                if [[ -d "$modules_dir" ]]; then
                  find "$modules_dir" -maxdepth 1 -mindepth 1 -type d -printf '%f\n' | head -1
                fi
              }

              get_systemd_version() {
                local systemd_path="$1"
                local base
                base=$(basename "$systemd_path")
                echo "''${base#*-systemd-}"
              }

              normalize_version() {
                local ver="$1"
                echo "''${ver//-rc/.}"
              }

              is_upgrade() {
                local old="$1"
                local new="$2"

                local old_norm new_norm
                old_norm=$(normalize_version "$old")
                new_norm=$(normalize_version "$new")

                if [[ "$old" == *"-rc"* && "$new" != *"-rc"* ]]; then
                  new_norm="''${new_norm}.0"
                elif [[ "$new" == *"-rc"* && "$old" != *"-rc"* ]]; then
                  old_norm="''${old_norm}.0"
                fi

                IFS='.' read -ra OLD_PARTS <<< "$old_norm"
                IFS='.' read -ra NEW_PARTS <<< "$new_norm"

                local max_len=''${#OLD_PARTS[@]}
                [[ ''${#NEW_PARTS[@]} -gt $max_len ]] && max_len=''${#NEW_PARTS[@]}

                for ((i=0; i<max_len; i++)); do
                  local o="''${OLD_PARTS[i]:-0}"
                  local n="''${NEW_PARTS[i]:-0}"
                  if [[ "$n" -gt "$o" ]]; then
                    return 0
                  elif [[ "$n" -lt "$o" ]]; then
                    return 1
                  fi
                done
                return 1
              }

              check_upgrades() {
                local reason=""

                if [[ -L "''${OLD_SYSTEM_PATH}/kernel" && -L "''${NEW_SYSTEM_PATH}/kernel" ]]; then
                  local old_kernel_store new_kernel_store
                  old_kernel_store=$(readlink -f "''${OLD_SYSTEM_PATH}/kernel")
                  new_kernel_store=$(readlink -f "''${NEW_SYSTEM_PATH}/kernel")

                  old_kernel_store=$(echo "$old_kernel_store" | cut -d'/' -f1-4)
                  new_kernel_store=$(echo "$new_kernel_store" | cut -d'/' -f1-4)

                  if [[ "$old_kernel_store" != "$new_kernel_store" ]]; then
                    local old_ver new_ver
                    old_ver=$(get_kernel_version "$old_kernel_store")
                    new_ver=$(get_kernel_version "$new_kernel_store")

                    if [[ -n "$old_ver" && -n "$new_ver" && "$old_ver" != "$new_ver" ]]; then
                      if is_upgrade "$old_ver" "$new_ver"; then
                        reason+="Linux Kernel (''${old_ver} -> ''${new_ver})"$'\n'
                      fi
                    fi
                  fi
                fi

                if [[ -L "''${OLD_SYSTEM_PATH}/systemd" && -L "''${NEW_SYSTEM_PATH}/systemd" ]]; then
                  local old_systemd_store new_systemd_store
                  old_systemd_store=$(readlink -f "''${OLD_SYSTEM_PATH}/systemd")
                  new_systemd_store=$(readlink -f "''${NEW_SYSTEM_PATH}/systemd")

                  if [[ "$old_systemd_store" != "$new_systemd_store" ]]; then
                    local old_ver new_ver
                    old_ver=$(get_systemd_version "$old_systemd_store")
                    new_ver=$(get_systemd_version "$new_systemd_store")

                    if [[ -n "$old_ver" && -n "$new_ver" && "$old_ver" != "$new_ver" ]]; then
                      if is_upgrade "$old_ver" "$new_ver"; then
                        reason+="Systemd (''${old_ver} -> ''${new_ver})"$'\n'
                      fi
                    fi
                  fi
                fi

                echo -n "$reason"
              }

              if [[ -f "$NIXOS_NEEDS_REBOOT" ]]; then
                cat "$NIXOS_NEEDS_REBOOT"
                exit 2
              fi

              old_version=$(cat "''${OLD_SYSTEM_PATH}/nixos-version" 2>/dev/null || echo "")
              new_version=$(cat "''${NEW_SYSTEM_PATH}/nixos-version" 2>/dev/null || echo "")

              if [[ "$old_version" == "$new_version" ]]; then
                echo "DEBUG: you are using the latest NixOS generation, no need to reboot" >&2
                exit 0
              fi

              reason=$(check_upgrades)

              if [[ -z "$reason" ]]; then
                echo "DEBUG: no updates available, moar uptime!!!" >&2
                exit 0
              fi

              if [[ "$DRY_RUN" == "true" ]]; then
                echo -n "$reason"
              else
                echo -n "$reason" > "$NIXOS_NEEDS_REBOOT"
              fi
              exit 2
            '';
          };
        });

      overlays.default = final: prev: {
        nixos-needsreboot = self.packages.${prev.system}.nixos-needsreboot;
      };
    };
}