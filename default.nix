{
  system ? builtins.currentSystem,
  pins ? import ./npins,
  pkgs ? import pins.nixpkgs { inherit system; },
}:
let
  craneLib = import (pins.crane + /default.nix) { inherit pkgs; };
  cfLib = import (pins.cf + /libpkgs.nix) pkgs;

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
        --prefix PATH ${pkgs.lib.makeBinPath [ pkgs.nix ]}
    '';
    postInstall = ''
      mkdir ./manpages ./completions
      $out/bin/nie man ./manpages
      $out/bin/nie completions ./completions
      installManPage ./manpages/*
      installShellCompletion completions/nie.{bash,fish,zsh}
    '';
  });

  checks = {
    default = checks.package;
    package = packages.default;
    clippy = craneLib.cargoClippy (commonArgs //{
      inherit cargoArtifacts;
      cargoClippyExtraArgs = "--all-targets -- --deny warnings";
    });
    statix = cfLib.mkStatixCheck { src = ./.; };
    deadnix = cfLib.mkDeadnixCheck { src = ./.; };
  };
}
