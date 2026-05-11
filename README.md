# nixos-needsreboot

Checks if you should reboot your NixOS machine after an upgrade brought in new goodies.

This is a pure Nix/bash implementation - no Rust toolchain required, no additional dependencies.

## What it does

1. Compares the booted system (`/run/booted-system`) against the current profile (`/nix/var/nix/profiles/system`)
2. If they differ, checks whether kernel or systemd have been **upgraded** (ignores downgrades)
3. Creates `/var/run/reboot-required` with the reason if a reboot is needed
4. Exits with code 2 if reboot required, 0 otherwise

## Usage
```bash
# Check if reboot needed (requires root)
sudo nixos-needsreboot

# Dry run - prints to stdout, no root required
nixos-needsreboot --dry-run

# Show version
nixos-needsreboot --version
```

## Installation

Add to your `flake.nix`:
```nix
{
  inputs.nixos-needsreboot.url = "github:harryprayiv/nixos-needsreboot";

  outputs = { self, nixpkgs, nixos-needsreboot, ... }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ({ pkgs, ... }: {
          environment.systemPackages = [
            nixos-needsreboot.packages.${pkgs.system}.default
          ];
        })
      ];
    };
  };
}
```

Or use the overlay:
```nix
{
  nixpkgs.overlays = [ nixos-needsreboot.overlays.default ];
  environment.systemPackages = [ pkgs.nixos-needsreboot ];
}
```

## Exit codes

- `0` - No reboot required
- `1` - Error (not root, not NixOS)
- `2` - Reboot required

## License

MIT