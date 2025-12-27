{
  description = "Run derivations from Nix files";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    cf.url = "github:jzbor/cornflakes";
  };

  outputs = { self, nixpkgs, cf, crane, ... }: (cf.mkLib nixpkgs).flakeForDefaultSystems (system:
  let
    pkgs = nixpkgs.legacyPackages.${system};
    craneLib = crane.mkLib pkgs;
  in {
    packages.default = craneLib.buildPackage rec {
      src = craneLib.cleanCargoSource ./.;
      strictDeps = true;

      cargoArtifacts = craneLib.buildDepsOnly {
        inherit src strictDeps;
      };

      nativeBuildInputs = with pkgs; [
        makeWrapper
        installShellFiles
      ];
      postFixup = ''
        wrapProgram $out/bin/niz \
          --set PATH ${pkgs.lib.makeBinPath [ pkgs.nix ]}
      '';
      postInstall = ''
        mkdir ./manpages ./completions
        $out/bin/niz man ./manpages
        $out/bin/niz completions ./completions
        installManPage ./manpages/*
        installShellCompletion completions/niz.{bash,fish,zsh}
      '';
    };

    devShells.default = craneLib.devShell {
      inherit (self.packages.${system}.default) name;

      # Additional tools
      nativeBuildInputs = [];
    };
  });
}
