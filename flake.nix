{
  description = "Run derivations from Nix files";

  outputs = _: {
    packages.x86_64-linux.default = (import ./default.nix { system = "x86_64-linux"; }).packages.default;
    packages.aarch64-linux.default = (import ./default.nix { system = "aarch64-linux"; }).packages.default;
    devShells.x86_64-linux.default = (import ./default.nix { system = "x86_64-linux"; }).devShells.default;
    devShells.aarch64-linux.default = (import ./default.nix { system = "aarch64-linux"; }).devShells.default;
  };
}
