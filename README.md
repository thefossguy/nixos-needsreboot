# nixos-needsreboot

Checks if you should reboot your NixOS machine in case an upgrade brought in
some new goodies. :)

# Usage as a flake

[![FlakeHub](https://img.shields.io/endpoint?url=https://flakehub.com/f/thefossguy/nixos-needsreboot/badge)](https://flakehub.com/flake/thefossguy/nixos-needsreboot)

Add nixos-needsreboot to your `flake.nix`:

```nix
{
  inputs.nixos-needsreboot.url = "https://flakehub.com/f/thefossguy/nixos-needsreboot/*.tar.gz";

  outputs = { self, nixos-needsreboot }: {
    # Use in your outputs
  };
}
```
