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
                      else mapAttrs (_: v: discover v (depth + 1)) set);
  in  if !result.success
      then "<broken>"
      else result.value;
in
  toJSON (discover (import path {}) 0)
