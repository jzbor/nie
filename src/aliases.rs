use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use std::sync::RwLock;

use crate::error::NieResult;
use crate::location::NixReference;

static CACHED_ALIASES: RwLock<Option<HashMap<String, NixReference>>> = RwLock::new(None);

const XDG_PREFIX: &str = "nie";

pub fn aliases() -> Option<HashMap<String, NixReference>> {
    CACHED_ALIASES.read().unwrap().as_ref().cloned()
}


pub fn load_aliases() -> NieResult<HashMap<String, NixReference>> {
    if let Some(aliases) = CACHED_ALIASES.read().unwrap().as_ref() {
        return Ok(aliases.to_owned());
    }

    let xdg_dirs = xdg::BaseDirectories::with_prefix(XDG_PREFIX);

    let mut map = HashMap::default();

    for file in xdg_dirs.find_config_files("aliases.txt") {
        let contents = fs::read_to_string(file)?;
        let pairs: Vec<_> = contents.lines()
            .flat_map(|l| l.split_once(' '))
            .map(|(k, v)| NixReference::from_str(v).map(|v| (k.to_owned(), v)))
            .collect::<NieResult<_>>()?;
        map.extend(pairs);
    }

    *CACHED_ALIASES.write().unwrap() = Some(map.clone());

    Ok(map)
}
