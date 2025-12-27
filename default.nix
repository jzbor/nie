{
  system ? builtins.currentSystem,
  pins ? import ./npins,
  pkgs ? import pins.nixpkgs { inherit system; },
}:
let
  craneLib = (import (pins.crane + /default.nix) { inherit pkgs; });

  nixFilter = path: _type: builtins.match ".*nix$" path != null;
  cargoFilter = path: type: (nixFilter path type) || (craneLib.filterCargoSources path type);

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
  commonArgs = {
    pname = "nie";

    src = pkgs.lib.cleanSourceWith {
      src = ./.;
      filter = cargoFilter;
      name = "source";
    };

    strictDeps = true;
    inherit cargoArtifacts;
  };
in rec {
  packages.default = craneLib.buildPackage (commonArgs // {
    nativeBuildInputs = with pkgs; [
      makeWrapper
      installShellFiles
    ];
    postFixup = ''
      wrapProgram $out/bin/nie \
        --set PATH ${pkgs.lib.makeBinPath [ pkgs.nix pkgs.git]}
    '';
    postInstall = ''
      mkdir ./manpages ./completions
      $out/bin/nie man ./manpages
      $out/bin/nie completions ./completions
      installManPage ./manpages/*
      installShellCompletion completions/nie.{bash,fish,zsh}
    '';
  });

  devShells.default = craneLib.devShell (commonArgs // {
    inherit (packages.default) name;

    # Additional tools
    nativeBuildInputs = [];
  });
}
