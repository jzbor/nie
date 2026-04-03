use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use crate::location::*;


/// Location specification for a remote or local repository or directory
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RepositoryLocation {
    Git(String),
    LocalFile(PathBuf),
    Tarball(String),
    Forgejo(String, String, String, Option<String>),
    Codeberg(String, String, Option<String>),
    Github(String, String, Option<String>),
}


impl FromStr for RepositoryLocation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(rest) = s.strip_prefix(RES_PREFIX_CODEBERG) {
            let (owner, mut repo) = rest.split_once('/')
                .ok_or(())?;
            let mut gitref = None;

            if let Some((r, g)) = repo.split_once('/') {
                repo = r;
                gitref = Some(g.to_owned());
            }

            Ok(RepositoryLocation::Codeberg(owner.to_owned(), repo.to_owned(), gitref))
        } else if let Some(rest) = s.strip_prefix(RES_PREFIX_GITHUB) {
            let (owner, mut repo) = rest.split_once('/')
                .ok_or(())?;
            let mut branch = None;

            if let Some((r, b)) = repo.split_once('/') {
                repo = r;
                branch = Some(b.to_owned());
            }

            Ok(RepositoryLocation::Github(owner.to_owned(), repo.to_owned(), branch))
        } else if let Some(rest) = s.strip_prefix(RES_PREFIX_FORGEJO) {
            let (domain, rest) = rest.split_once('/')
                .ok_or(())?;
            let (owner, mut repo) = rest.split_once('/')
                .ok_or(())?;
            let mut gitref = None;

            if let Some((r, g)) = repo.split_once('/') {
                repo = r;
                gitref = Some(g.to_owned());
            }

            Ok(RepositoryLocation::Forgejo(domain.to_owned(), owner.to_owned(), repo.to_owned(), gitref))
        } else if let Some(path) = s.strip_prefix(RES_PREFIX_LOCAL) {
            Ok(RepositoryLocation::LocalFile(PathBuf::from(path)))
        } else if let Some(repo) = s.strip_prefix(RES_PREFIX_GIT) {
            Ok(RepositoryLocation::Git(repo.to_owned()))
        } else if s.starts_with(RES_PREFIX_TAR) && RES_SUFFIXES_TAR.iter().any(|suf| s.ends_with(suf)) {
            Ok(RepositoryLocation::Tarball(s.to_owned()))
        } else if PathBuf::from(s).is_dir() && !PathBuf::from(s).join(".git").is_dir() {
            Ok(RepositoryLocation::LocalFile(PathBuf::from(s)))
        } else {
            Ok(RepositoryLocation::Git(s.to_owned()))
        }
    }
}

impl Display for RepositoryLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RepositoryLocation::*;
        match self {
            LocalFile(path) => write!(f, "{}{}", RES_PREFIX_LOCAL, path.to_string_lossy()),
            Git(url) => write!(f, "{}{}", RES_PREFIX_GIT, url),
            Tarball(url) => write!(f, "tar://{}", url),
            Forgejo(domain, owner, repo, gitref) => write!(f, "{}{}/{}/{}{}", RES_PREFIX_FORGEJO, domain, owner, repo,
                gitref.as_ref().map(|b| format!("/{}", b)).unwrap_or_default()),
            Codeberg(owner, repo, gitref) => write!(f, "{}{}/{}{}", RES_PREFIX_CODEBERG, owner, repo,
                gitref.as_ref().map(|b| format!("/{}", b)).unwrap_or_default()),
            Github(owner, repo, branch) => write!(f, "{}{}/{}{}", RES_PREFIX_GITHUB, owner, repo,
                branch.as_ref().map(|b| format!("/{}", b)).unwrap_or_default()),
        }
    }
}

impl Default for RepositoryLocation {
    fn default() -> Self {
        RepositoryLocation::Git("./.".to_owned())
    }
}
