use std::collections::BTreeMap;
use std::fmt::Display;
use std::path::PathBuf;

use crate::error::{NieError, NieResult};
use crate::location::{NixFileReference, RepositoryLocation};


#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct RepositoryReference {
    location: RepositoryLocation,
    fetch_args: BTreeMap<String, String>,
}

/// Reference to a repository consisting of a [`RepositoryLocation`] and a collection of additional
/// arguments used to fetch the repository.
impl RepositoryReference {
    pub fn location(&self) -> &RepositoryLocation {
        &self.location
    }

    pub fn location_mut(&mut self) -> &mut RepositoryLocation {
        &mut self.location
    }

    pub fn fetch_args(&self) -> &BTreeMap<String, String> {
        &self.fetch_args
    }

    pub fn fetch_args_mut(&mut self) -> &mut BTreeMap<String, String> {
        &mut self.fetch_args
    }

    pub fn fetch_args_json(&self) -> NieResult<BTreeMap<String, serde_json::Value>> {
        self.fetch_args.iter()
            .map(|(k, v)| serde_json::from_str(v).map(|v| (k.clone(), v)))
            .collect::<serde_json::Result<_>>()
            .map_err(NieError::Json)
    }

    pub fn with_filename(&self, filename: Option<PathBuf>) -> NixFileReference {
        NixFileReference::new(self.clone(), filename)
    }
}


impl Display for RepositoryReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.location())?;
        for (k, v) in &self.fetch_args {
            write!(f, "#{}={}", k, v)?;
        }
        Ok(())
    }
}

