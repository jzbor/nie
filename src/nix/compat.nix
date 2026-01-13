{ path }:

let
  flake-compat = builtins.fetchTarball {
    url = "https://git.lix.systems/lix-project/flake-compat/archive/549f2762aebeff29a2e5ece7a7dc0f955281a1d1.tar.gz";
    sha256 = "0g4izwn5k7qpavlk3w41a92rhnp4plr928vmrhc75041vzm3vb1l";
  };
  flake = import flake-compat {
    src = path;
    copySourceTreeToStore = false;  # paths already in store
  };
in
  flake.outputs
