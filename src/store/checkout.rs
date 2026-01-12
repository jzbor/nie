use std::path::PathBuf;
use std::sync::Arc;

use crate::error::NieResult;
use crate::interaction::announce;
use crate::location::{RepositoryLocation, RepositoryReference};
use crate::store::file::NixFile;
use crate::nix;
use crate::registry::Registry;


static CHECKOUT_REGISTRY: Registry<RepositoryReference, Checkout> = Registry::new();

#[derive(Clone)]
pub struct Checkout(Arc<InnerCheckout>);

struct InnerCheckout {
    repository: RepositoryReference,
    path: PathBuf,
}

impl Checkout {
    pub fn create(repository: RepositoryReference) -> NieResult<Self> {
        if let Some(checkout) = CHECKOUT_REGISTRY.lookup(&repository) {
            return Ok(checkout);
        }

        announce(&format!("Fetching {}", repository.location()));

        use RepositoryLocation::*;
        let path = match repository.location() {
            Git(url) => nix::fetch_git(url, repository.checkout_args())?,
            Tarball(url) => nix::fetch_tarball(url, repository.checkout_args())?,
            Codeberg(owner, repo, gitref) => nix::fetch_codeberg(owner, repo,
                gitref.as_ref().map(|s| s.as_str()), repository.checkout_args())?,
            Github(owner, repo, branch) => nix::fetch_github(owner, repo,
                branch.as_ref().map(|s| s.as_str()), repository.checkout_args())?,
        };

        let checkout = Checkout(Arc::new(InnerCheckout {
            repository: repository.clone(),
            path
        }));

        CHECKOUT_REGISTRY.store(repository.clone(), checkout.clone());

        Ok(checkout)
    }

    pub fn create_all(repositories: impl IntoIterator<Item = RepositoryReference>) -> NieResult<Vec<Self>> {
        repositories.into_iter()
            .map(Self::create)
            .collect()
    }

    pub fn file(&self, filename: Option<PathBuf>, force_flake_compat: bool) -> NieResult<NixFile> {
        NixFile::new(self.clone(), filename, force_flake_compat)
    }

    pub fn files(files: impl IntoIterator<Item = (Self, Option<PathBuf>)>, force_flake_compat: bool)
            -> NieResult<Vec<NixFile>> {
        files.into_iter()
            .map(|(c, f)| c.file(f.clone(), force_flake_compat))
            .collect()
    }

    pub fn repository(&self) -> &RepositoryReference {
        &self.0.repository
    }

    pub fn path(&self) -> &PathBuf {
        &self.0.path
    }
}

