{
  system ? builtins.currentSystem,
  pins ? import ./npins,
  pkgs ? import pins.nixpkgs { inherit system; },
  docker-nixpkgs ? pins.docker-nixpkgs { inherit pkgs; },
}:
let
  craneLib = import (pins.crane + /default.nix) { inherit pkgs; };
  cfLib = import (pins.cf + /libpkgs.nix) pkgs;

  nixFilter = path: _type: builtins.match ".*nix$" path != null;
  readmeFilter = path: _type: builtins.match ".*README.md$" path != null;
  cargoFilter = path: type: (nixFilter path type) || (readmeFilter path type) || (craneLib.filterCargoSources path type);

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

  buildImageWithNix = import ("${docker-nixpkgs}" + "/images/nix/default.nix");
in rec {
  packages = {
    inherit (pkgs) npins;

    default = packages.nie;
    nie = craneLib.buildPackage (commonArgs // {
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

    nieImage = pkgs.dockerTools.buildImage {
      name = "nie";
      tag = "latest";

      fromImage = buildImageWithNix {
        inherit (pkgs) dockerTools bashInteractive cacert coreutils curl gnutar gzip iana-etc nix openssh xz;

        # We are actually going to use Git so we use the full version.
        gitReallyMinimal = pkgs.git;

        extraContents = [
          packages.nie
        ];
      };

      config.Cmd = [ "/bin/bash" ];
    };

    # nieImage = pkgs.dockerTools.buildImage {
    #   name = "nie";
    #   tag = "latest";

    #   fromImage = packages.nixImage;
    #   # fromImage = nix-actions-container.packages.${system}.default;
    #   copyToRoot = pkgs.buildEnv {
    #     name = "image-root";
    #     paths = with pkgs; [
    #       packages.nie
    #       bash
    #     ];
    #     pathsToLink = [ "/bin" ];
    #   };

    #   config.Cmd = [ "/bin/bash" ];
    # };
  };

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

  devShells.default = pkgs.mkShellNoCC {
    inherit (packages.default) name;
    packages = with pkgs; [
      npins
      nix-prefetch-docker
    ];
  };
}
