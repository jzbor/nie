use std::path::PathBuf;
use std::sync::Arc;

use crate::error::NieResult;
use crate::file::NixFile;
use crate::interaction::announce;
use crate::location::{RepositoryLocation, RepositoryReference};
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

    pub fn file(&self, filename: Option<PathBuf>) -> NieResult<NixFile> {
        NixFile::new(self.clone(), filename)
    }

    pub fn files(files: impl IntoIterator<Item = (Self, Option<PathBuf>)>) -> NieResult<Vec<NixFile>> {
        files.into_iter()
            .map(|(c, f)| c.file(f.clone()))
            .collect()
    }

    pub fn repository(&self) -> &RepositoryReference {
        &self.0.repository
    }

    pub fn path(&self) -> &PathBuf {
        &self.0.path
    }
}

