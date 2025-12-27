use std::iter;
use std::path::PathBuf;

use crate::checkout::Checkout;
use crate::error::NieResult;
use crate::file::NixFile;
use crate::interaction::announce;
use crate::location::NixReference;
use crate::nix;


#[derive(clap::Args)]
pub struct ShellCommand {
    /// Nix references to fetch and add to shell
    refs: Vec<NixReference>,

    /// Additional arguments for the nix builder (see nix-build(1))
    #[clap(last = true)]
    nix_args: Vec<String>,
}

impl super::Command for ShellCommand {
    fn exec(self) -> NieResult<()> {
        let repo_refs = self.refs.iter().map(|s| s.repository()).cloned();
        let filenames = self.refs.iter().map(|s| s.filename().cloned());
        let attributes = self.refs.iter().map(|s| s.attribute()).cloned();
        let checkouts = Checkout::create_all(repo_refs)?;
        let files = Checkout::files(iter::zip(checkouts.iter().cloned(), filenames))?;
        let outputs = NixFile::outputs(iter::zip(files.iter().cloned(), attributes))?;
        let paths: Vec<PathBuf> = outputs.into_iter()
            .map(|o| o.build(true, &self.nix_args))
            .collect::<NieResult<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();

        announce(&format!("Entering shell with {} added paths", paths.len()));
        nix::shell(&paths, &self.nix_args)
    }
}
