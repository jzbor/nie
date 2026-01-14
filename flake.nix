{
  outputs = _: (import ((import ./npins).cf + /lib.nix)).mkCompatFlake (import ./default.nix) [
      "x86_64-linux"
      "aarch64-linux"
      "aarch64-darwin"
  ];
}
