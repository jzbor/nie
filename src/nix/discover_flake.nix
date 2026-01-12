{ path, maxdepth }:

with builtins;
let
  isDerivation = value: value.type or null == "derivation";
  isModule = value: value ? imports;
  isConfig = value: value._type or null == "configuration";
  isSet = value: typeOf value == "set";
  discover = set: depth: let
    result = tryEval (if !(isSet set) || isDerivation set || isModule set || isConfig set || depth >= maxdepth
                      then {}
                      else mapAttrs (n: v: discover v (depth + 1)) set);
  in  if !result.success
      then "<broken>"
      else result.value;
  flake-compat = builtins.fetchTarball {
    url = "https://git.lix.systems/lix-project/flake-compat/archive/549f2762aebeff29a2e5ece7a7dc0f955281a1d1.tar.gz";
    sha256 = "0g4izwn5k7qpavlk3w41a92rhnp4plr928vmrhc75041vzm3vb1l";
  };
  flake = import flake-compat { src = path; };
  # items = foldl' (acc: elem: acc // elem) {} (map (name: {
  #   ${name} = (discover flake.outputs.${name} 1);
  # })(attrNames flake.outputs));
  items = map (name: {
    inherit name;
    value = (discover flake.outputs.${name} 1);
  }) (attrNames flake.outputs);
in
  toJSON items
