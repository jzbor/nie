# Nie
`nie` is a `nix`-wrapper that allows you to build, run or load your derivations similar to the `nix3` command-line interface and flakes.
It does not dependent on Flakes or any other experimental features and relies solely on the stable `nix2` interface.

You can try out the program in a temporary shell:
```sh
# on a Flake-enabled system:
nix shell git+https://codeberg.org/jzbor/nie.git

# on any other machine with Nix installed
nix-shell -E '(import (builtins.fetchGit { url = "https://codeberg.org/jzbor/nie"; }) {}).packages.default'

# on a machine with nie installed
nie shell codeberg://jzbor/nie
```


## Features
- Fetch repositories from Codeberg, Github and other Git sources.
- Discover packages and checks from `default.nix`, `flake.nix` or a custom Nix file.
- Add programs to your shell and enter development shells.
- Pin shells for automatic or offline usage.
- Compatibility with Flakes is enabled by [lix-project/flake-compat](https://https://git.lix.systems/lix-project/flake-compat)

## Planned Features
- [ ] Support for Forgejo instances other than https://codeberg.org
- [ ] Support for Gitlab instances
- [ ] Improved user feedback/output messages

## Non-Goals
- NixOS/HomeManager integration: You can use `nixos-rebuild`, `home-manager`, [`nh`](https://github.com/nix-community/nh) or custom scripts for those.
- Input pinning: Use [`npins`](https://github.com/andir/npins), [`niv`](https://github.com/nmattia/niv) or [`Nixtamal`](https://nixtamal.toast.al/).
