//! Alias caching and management

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::RwLock;

use crate::error::NieResult;
use crate::location::NixReference;


/// XDG subdirectory for this program
const XDG_PREFIX: &str = "nie";

/// Alias cache created by [`load_aliases()`]
static CACHED_ALIASES: RwLock<Option<HashMap<String, NixReference>>> = RwLock::new(None);


/// Returns a [`HashMap`] that maps alias names to their respective [`NixReference`], returns
/// [`None`] if aliases have not been loaded yet via [`load_aliases()`]
pub fn aliases() -> Option<HashMap<String, NixReference>> {
    CACHED_ALIASES.read().unwrap().as_ref().cloned()
}

/// Returns the path to the users `aliases.txt` file
///
/// Parent directories are created via [`xdg::BaseDirectories::place_config_file()`].
pub fn user_alias_file() -> NieResult<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(XDG_PREFIX);
    let path = xdg_dirs.place_config_file("aliases.txt")?;
    Ok(path)
}

/// Returns a [`Vec`] with all relevant `aliases.txt` files (system, user, etc.) according to
/// [`xdg::BaseDirectories::find_config_files()`].
pub fn alias_files() -> Vec<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(XDG_PREFIX);
    xdg_dirs.find_config_files("aliases.txt")
        .collect()
}

/// Populates [`CACHED_ALIASES`] from [`alias_files()`] if necessary and return all aliases as [`HashMap`] mapping the
/// alias name to its respective [`NixReference`]
pub fn load_aliases() -> NieResult<HashMap<String, NixReference>> {
    // Return aliases if already cached
    if let Some(aliases) = aliases() {
        return Ok(aliases);
    }

    // Load and parse all alias files
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

    // Populate cache
    *CACHED_ALIASES.write().unwrap() = Some(map.clone());

    Ok(map)
}
