use std::path::PathBuf;
use std::sync::Arc;

use crate::error::{NieError, NieResult};
use crate::interact::*;
use crate::location::{RepositoryLocation, RepositoryReference};
use crate::store::file::NixFile;
use crate::{EvalArgs, nix};
use crate::registry::Registry;


/// Registry to cache known, already fetched [`Checkout`]s
static CHECKOUT_REGISTRY: Registry<RepositoryReference, Checkout> = Registry::new();


/// Local checkout of a repository
///
/// Derived from a [`RepositoryReference`] (see [`Checkout::fetch()`] and [`Checkout::fetch_all()`])
#[derive(Clone)]
pub struct Checkout(Arc<InnerCheckout>);

struct InnerCheckout {
    repository: RepositoryReference,
    path: PathBuf,
}


impl Checkout {
    fn new(repository: RepositoryReference, path: PathBuf) -> Self {
        Checkout(Arc::new(InnerCheckout { repository, path }))
    }

    /// Fetch a repository into the local store if necessary and create a new [`Checkout`] from it.
    pub fn fetch(repository: RepositoryReference) -> NieResult<Self> {
        Self::fetch_all([repository])
            .map(|p| p.into_iter().next().unwrap())
    }

    /// Fetch all repositories into the local store if necessary and create new [`Checkout`]s from it.
    ///
    /// See also [`nix::fetch_local()`] and [`nix::fetch_all()`]
    pub fn fetch_all(repositories: impl IntoIterator<Item = RepositoryReference>) -> NieResult<Vec<Self>> {
        use RepositoryLocation::*;
        let repositories: Vec<_> = repositories.into_iter().collect();
        inform_fetch_multiple(&repositories);

        let (known, unknown): (Vec<_>, Vec<_>) = repositories.into_iter()
                                   .enumerate()
                                   .partition(|(_, r)| CHECKOUT_REGISTRY.lookup(r).is_some());
        let (unknown_local, unknown_other): (Vec<_>, Vec<_>) = unknown.into_iter()
                                   .partition(|(_, r)| matches!(r.location(), LocalFile(..)));

        let resolved_known: Vec<_> = known.into_iter()
            .map(|(i, r)| (i, CHECKOUT_REGISTRY.lookup(&r).unwrap()))
            .collect();

        let resolved_local: Vec<_> = unknown_local.into_iter()
            .map(|(i, r)| if let LocalFile(path) = r.location() {
                nix::fetch_local(path, r.fetch_args())
                    .map_err(|_| NieError::FetchFailure(r.clone()))
                    .map(|p| (i, p, r))
            } else {
                panic!()
            })
            .map(|res| res.map(|(i, p, r)| (i, Checkout::new(r, p))))
            .collect::<NieResult<_>>()?;

        let (unknown_other_idx, unknown_other_rep): (Vec<_>, Vec<_>) = unknown_other.into_iter().unzip();
        let fetched_other = nix::fetch_all(&unknown_other_rep)
            .map_err(|_| if unknown_other_rep.len() == 1 {
                NieError::FetchFailure(unknown_other_rep[0].clone())
            } else {
                NieError::FetchFailureMultiple(unknown_other_rep.len())
            })?;
        let resolved_other: Vec<_> = unknown_other_idx.into_iter()
            .zip(fetched_other)
            .zip(unknown_other_rep)
            .map(|((i, p), r)| (i, p, r))
            .map(|(i, p, r)| (i, Checkout::new(r, p)))
            .collect();

        resolved_local.iter().for_each(|(_, c)| CHECKOUT_REGISTRY.store(c.repository().clone(), c.clone()));
        resolved_other.iter().for_each(|(_, c)| CHECKOUT_REGISTRY.store(c.repository().clone(), c.clone()));

        let mut all = Vec::new();
        all.extend(resolved_known);
        all.extend(resolved_local);
        all.extend(resolved_other);

        all.sort_by_key(|e| e.0);

        Ok(all.into_iter().map(|(_, c)| c).collect())
    }

    /// Creates a new [`NixFile`] from a file in this [`Checkout`].
    pub fn file(&self, filename: Option<PathBuf>, eval_args: EvalArgs) -> NieResult<NixFile> {
        NixFile::new(self.clone(), filename, eval_args)
    }

    /// Creates a new [`NixFile`]s from an iterator over [`Checkout`]s and paths.
    pub fn files(files: impl IntoIterator<Item = (Self, Option<PathBuf>)>, eval_args: EvalArgs)
            -> NieResult<Vec<NixFile>> {
        files.into_iter()
            .map(|(c, f)| c.file(f.clone(), eval_args.clone()))
            .collect()
    }

    pub fn repository(&self) -> &RepositoryReference {
        &self.0.repository
    }

    pub fn path(&self) -> &PathBuf {
        &self.0.path
    }
}

