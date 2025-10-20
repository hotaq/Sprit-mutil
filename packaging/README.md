# Packaging

This directory contains packaging files for different platforms and package managers.

## Files

- **PKGBUILD** - Arch Linux AUR package build script
- **flake.nix** - Nix/NixOS package definition

## Installation

### Arch Linux (AUR)
```bash
# Using paru
paru -U PKGBUILD

# Using yay
yay -U PKGBUILD

# Manual build
makepkg -si
```

### Nix/NixOS
```bash
# Using the flake
nix build .

# Install from flake
nix profile install .

# In NixOS configuration
environment.systemPackages = [
  (import (builtins.fetchGit {
    url = "https://github.com/hotaq/Sprit-mutil";
    ref = "main";
  }) {}).packages.x86_64-linux.default
];
```