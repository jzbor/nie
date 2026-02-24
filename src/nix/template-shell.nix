{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    # commands to be executed on entering
    shellHook = '''';

    # build tools
    nativeBuildInputs = with pkgs; [];

    # libraries/dependencies
    buildInputs = with pkgs; [];
}
