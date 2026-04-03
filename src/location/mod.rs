//! Data structures for describing the location of Nix outputs
//!
//! A [`NixReference`] contains a [`NixFileReference`] which contains a [`RepositoryReference`]
//! which contains a [`RepositoryLocation`].
pub use attribute_path::AttributePath;
pub use reference::NixReference;
pub use file_reference::NixFileReference;
pub use repo_reference::RepositoryReference;
pub use repo_location::RepositoryLocation;

mod attribute_path;
mod reference;
mod file_reference;
mod repo_reference;
mod repo_location;


const RES_PREFIX_CODEBERG: &str = "codeberg://";
const RES_PREFIX_FORGEJO: &str = "forgejo://";
const RES_PREFIX_GIT: &str = "git://";
const RES_PREFIX_GITHUB: &str = "github://";
const RES_PREFIX_LOCAL: &str = "file://";
const RES_PREFIX_TAR: &str = "https://";
const RES_SUFFIXES_TAR: &[&str] = &[ ".tar", ".tar.gz", ".tar.xz", ".tar.bz2" ];
