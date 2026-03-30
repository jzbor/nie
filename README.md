# Nie
`nie` is a `nix`-wrapper that allows you to build, run or load your derivations similar to the `nix3` command-line interface and flakes.
It is not dependent on Flakes or any other experimental features and relies solely on the stable `nix2` interface.

## Features
- Fetch repositories from Codeberg, Github and other Git sources.
- Discover packages and checks from `default.nix`, `flake.nix` or a custom Nix file.
- Add programs to your shell and enter development shells.
- Pin shells for automatic or offline usage.

## Planned Features
- [ ] Support for Forgejo instances other than https://codeberg.org
- [ ] Improved user feedback/output messages

## Non-Goals
- NixOS/HomeManager integration: You can use `nixos-rebuild`, `home-manager`, [`nh`](https://github.com/nix-community/nh) or custom scripts for those.
