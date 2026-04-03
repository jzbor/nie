use std::fmt::Display;
use std::ops::Deref;
use std::path::PathBuf;

use crate::location::{AttributePath, NixReference, RepositoryReference};


/// Reference to a Nix file consisting of a [`RepositoryReference`] (in turn consisting of a [`super::RepositoryLocation`]) and a file path.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct NixFileReference {
    repository: RepositoryReference,
    filename: Option<PathBuf>,
}


impl NixFileReference {
    pub fn new(repository: RepositoryReference, filename: Option<PathBuf>) -> Self {
        Self { repository, filename }
    }

    pub fn repository(&self) -> &RepositoryReference {
        &self.repository
    }

    pub fn repository_mut(&mut self) -> &mut RepositoryReference {
        &mut self.repository
    }

    pub fn filename(&self) -> Option<&PathBuf> {
        self.filename.as_ref()
    }

    pub fn filename_mut(&mut self) -> &mut Option<PathBuf> {
        &mut self.filename
    }

    /// Returns a copy of `self` with a new [`AttributePath`].
    pub fn with_attribute(&self, attr: AttributePath) -> NixReference {
        NixReference::new(self.clone(), attr)
    }
}


impl Display for NixFileReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.location())?;
        if let Some(file) = &self.filename {
            write!(f, "#file={}", file.to_string_lossy())?;
        }
        for (k, v) in self.fetch_args() {
            write!(f, "#{}={}", k, v)?;
        }
        Ok(())
    }
}

impl Deref for NixFileReference {
    type Target = RepositoryReference;

    fn deref(&self) -> &Self::Target {
        self.repository()
    }
}
