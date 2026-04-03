use std::collections::VecDeque;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

use crate::aliases;
use crate::error::NieError;
use crate::location::{AttributePath, NixFileReference, RepositoryLocation};


#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct NixReference {
    file: NixFileReference,
    attribute: AttributePath,
}


/// Full reference to a Nix output consisting of a [`NixFileReference`] (in turn consisting of a [`super::RepositoryReference`]) and an [`AttributePath`].
impl NixReference {
    pub fn new(file: NixFileReference, attribute: AttributePath) -> Self {
        Self { file, attribute }
    }

    pub fn file(&self) -> &NixFileReference {
        &self.file
    }

    pub fn attribute(&self) -> &AttributePath {
        &self.attribute
    }
}


impl Deref for NixReference {
    type Target = NixFileReference;

    fn deref(&self) -> &Self::Target {
        self.file()
    }
}

impl FromStr for NixReference {
    type Err = NieError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens: VecDeque<_> = s.split("#").collect();
        let url = tokens.pop_front()
            .ok_or(NieError::InvalidLocationSpec(s.to_owned()))?;

        let mut nref = NixReference::default();

        match aliases::aliases().and_then(|a| {
            a.get(url).cloned()
        }) {
            Some(new_ref) => {
                nref = new_ref;
            },
            None => {
                *nref.file.repository_mut().location_mut() = RepositoryLocation::from_str(url)
                    .map_err(|_| NieError::InvalidLocationSpec(s.to_owned()))?;
            }
        };

        for token in tokens {
            if let Some((k, v)) = token.split_once('=') {
                match k {
                    "f" | "file" => *nref.file.filename_mut() = Some(v.into()),
                    _ => {
                        nref.file.repository_mut().fetch_args_mut().insert(k.to_owned(), v.to_owned());
                    },
                }
            } else {
                nref.attribute = AttributePath::from_str(token)?;
            }
        }

        Ok(nref)
    }
}

impl Display for NixReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.location())?;
        if !self.attribute.is_toplevel() {
            write!(f, "#{}", self.attribute)?;
        }
        if let Some(file) = &self.filename() {
            write!(f, "#file={}", file.to_string_lossy())?;
        }
        for (k, v) in self.fetch_args() {
            write!(f, "#{}={}", k, v)?;
        }
        Ok(())

    }
}

