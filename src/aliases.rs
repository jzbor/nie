use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::RwLock;

use crate::error::NieResult;
use crate::location::NixReference;

static CACHED_ALIASES: RwLock<Option<HashMap<String, NixReference>>> = RwLock::new(None);

const XDG_PREFIX: &str = "nie";

pub fn aliases() -> Option<HashMap<String, NixReference>> {
    CACHED_ALIASES.read().unwrap().as_ref().cloned()
}


pub fn user_alias_file() -> NieResult<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(XDG_PREFIX);
    let path = xdg_dirs.place_config_file("aliases.txt")?;
    Ok(path)
}

pub fn alias_files() -> Vec<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(XDG_PREFIX);
    xdg_dirs.find_config_files("aliases.txt")
        .collect()
}

pub fn load_aliases() -> NieResult<HashMap<String, NixReference>> {
    if let Some(aliases) = CACHED_ALIASES.read().unwrap().as_ref() {
        return Ok(aliases.to_owned());
    }

    let mut map = HashMap::default();

    for file in alias_files() {
        let contents = fs::read_to_string(file)?;
        let pairs: Vec<_> = contents.lines()
            .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("//"))
            .flat_map(|l| l.split_once(' '))
            .map(|(k, v)| NixReference::from_str(v).map(|v| (k.to_owned(), v)))
            .collect::<NieResult<_>>()?;
        map.extend(pairs);
    }

    *CACHED_ALIASES.write().unwrap() = Some(map.clone());

    Ok(map)
}
