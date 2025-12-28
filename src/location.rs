use std::collections::{BTreeMap, VecDeque};
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::str::FromStr;

use crate::error::NieError;


#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct AttributePath(Vec<String>);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NixReference {
    file: NixFileReference,
    attribute: AttributePath,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RepositoryReference {
    location: RepositoryLocation,
    checkout_args: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NixFileReference {
    repository: RepositoryReference,
    filename: Option<PathBuf>,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RepositoryLocation {
    Git(String),
    Tarball(String),
    Codeberg(String, String, Option<String>),
    Github(String, String, Option<String>),
}


impl NixReference {
    pub fn file(&self) -> &NixFileReference {
        &self.file
    }

    pub fn attribute(&self) -> &AttributePath {
        &self.attribute
    }
}

impl NixFileReference {
    pub fn repository(&self) -> &RepositoryReference {
        &self.repository
    }

    pub fn filename(&self) -> Option<&PathBuf> {
        self.filename.as_ref()
    }

    pub fn with_attribute(&self, attr: AttributePath) -> NixReference {
        NixReference {
            file: self.clone(),
            attribute: attr,
        }
    }
}

impl RepositoryReference {
    pub fn location(&self) -> &RepositoryLocation {
        &self.location
    }

    pub fn checkout_args(&self) -> &BTreeMap<String, String> {
        &self.checkout_args
    }

    pub fn with_file(&self, filename: Option<PathBuf>) -> NixFileReference {
        NixFileReference {
            repository: self.clone(),
            filename
        }
    }
}

impl AttributePath {
    pub fn child(&self, name: String) -> Self {
        let mut new_path = self.0.clone();
        new_path.push(name);
        AttributePath(new_path)
    }

    pub fn depth(&self) -> usize {
        self.len() - 1
    }

    pub fn is_toplevel(&self) -> bool {
        self.is_empty()
    }

    pub fn name(&self) -> Option<&str> {
        self.last().map(|l| l.as_str())
    }

    pub fn to_string_user(&self) -> String {
        if self.is_toplevel() {
            String::from("<toplevel>")
        } else {
            self.to_string()
        }
    }
}



impl Deref for NixReference {
    type Target = NixFileReference;

    fn deref(&self) -> &Self::Target {
        self.file()
    }
}

impl Deref for NixFileReference {
    type Target = RepositoryReference;

    fn deref(&self) -> &Self::Target {
        self.repository()
    }
}

impl FromStr for NixReference {
    type Err = NieError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens: VecDeque<_> = s.split("#").collect();
        let url = tokens.pop_front()
            .ok_or(NieError::InvalidLocationSpec(s.to_owned()))?;
        let location = RepositoryLocation::from_str(url)
            .map_err(|_| NieError::InvalidLocationSpec(s.to_owned()))?;
        let mut attribute = AttributePath::default();
        let mut checkout_args = BTreeMap::default();
        let mut filename = None;

        for token in tokens {
            if let Some((k, v)) = token.split_once('=') {
                match k {
                    "f" | "file" => filename = Some(v.into()),
                    _ => { checkout_args.insert(k.to_owned(), v.to_owned()); },
                }
            } else {
                attribute = AttributePath::from_str(token)?;
            }
        }

        Ok(NixReference {
            file: NixFileReference {
                repository: RepositoryReference {
                    location,
                    checkout_args,
                },
                filename,
            },
            attribute
        })
    }
}

impl FromStr for RepositoryLocation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(rest) = s.strip_prefix("codeberg://") {
            let (owner, mut repo) = rest.split_once('/')
                .ok_or(())?;
            let mut gitref = None;

            if let Some((r, g)) = repo.split_once('/') {
                repo = r;
                gitref = Some(g.to_owned());
            }

            Ok(RepositoryLocation::Codeberg(owner.to_owned(), repo.to_owned(), gitref))
        } else if let Some(rest) = s.strip_prefix("github://") {
            let (owner, mut repo) = rest.split_once('/')
                .ok_or(())?;
            let mut branch = None;

            if let Some((r, b)) = repo.split_once('/') {
                repo = r;
                branch = Some(b.to_owned());
            }

            Ok(RepositoryLocation::Github(owner.to_owned(), repo.to_owned(), branch))
        } else if (s.starts_with("https://") || s.starts_with("http://"))
            && (s.ends_with(".tar.gz") || s.ends_with(".tag.xz") || s.ends_with(".tag.bz2")) {
            Ok(RepositoryLocation::Tarball(s.to_owned()))
        } else {
            Ok(RepositoryLocation::Git(s.to_owned()))
        }
    }
}

impl Display for NixReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.location)?;
        if !self.attribute.is_toplevel() {
            write!(f, "#{}", self.attribute)?;
        }
        if let Some(file) = &self.filename {
            write!(f, "#file:{}", file.to_string_lossy())?;
        }
        for (k, v) in &self.checkout_args {
            write!(f, "#{}:{}", k, v)?;
        }
        Ok(())

    }
}

impl Display for NixFileReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.location)?;
        if let Some(file) = &self.filename {
            write!(f, "#file:{}", file.to_string_lossy())?;
        }
        for (k, v) in &self.checkout_args {
            write!(f, "#{}:{}", k, v)?;
        }
        Ok(())
    }
}

impl Display for RepositoryReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.location)?;
        for (k, v) in &self.checkout_args {
            write!(f, "#{}:{}", k, v)?;
        }
        Ok(())
    }
}

impl Display for RepositoryLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RepositoryLocation::*;
        match self {
            Git(url) | Tarball(url) => write!(f, "{}", url),
            Codeberg(owner, repo, gitref) => write!(f, "codeberg://{}/{}{}", owner, repo,
                gitref.as_ref().map(|b| format!("/{}", b)).unwrap_or_default()),
            Github(owner, repo, branch) => write!(f, "github://{}/{}{}", owner, repo,
                branch.as_ref().map(|b| format!("/{}", b)).unwrap_or_default()),
        }
    }
}

impl Deref for AttributePath {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AttributePath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for AttributePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("."))
    }
}

impl FromStr for AttributePath {
    type Err = NieError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self::default())
        } else {
            Ok(AttributePath(s.split(".").map(|s| s.to_owned()).collect()))
        }
    }
}
