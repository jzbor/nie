{
  system ? builtins.currentSystem,
  overrideSources ? {},
  sources ? { nixpkgs = <nixpkgs>; } // overrideSources,
  # => You could also use [npins](https://github.com/andir/npins) for input pinning:
  # sources ? import ./npins // overrideSources,
  pkgs ? import sources.nixpkgs { inherit system; },
}: rec {
  packages.hello = pkgs.hello;

  devShells.default = pkgs.mkShellNoCC {
    name = "example";

    packages = with pkgs; [
      npins
      packages.hello
    ];
  };
}
