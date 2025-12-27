use std::iter;

use crate::checkout::Checkout;
use crate::error::NieResult;
use crate::location::NixReference;


#[derive(clap::Args)]
pub struct ShowCommand {
    /// Nix references to fetch and show
    refs: Vec<NixReference>,
}

impl super::Command for ShowCommand {
    fn exec(self) -> NieResult<()> {
        let repo_refs = self.refs.iter().map(|s| s.repository()).cloned();
        let filenames = self.refs.iter().map(|s| s.filename().cloned());
        let checkouts = Checkout::create_all(repo_refs)?;
        let files = Checkout::files(iter::zip(checkouts.iter().cloned(), filenames))?;

        println!();

        for (reference, file) in iter::zip(self.refs.into_iter(), files.into_iter()) {
            println!("[{}]:", reference);
            file.attributes()?
                .for_each(|a| println!("{:>width$}{}", "", a.name().unwrap_or_default(), width=(a.depth() + 1) * 2));
            println!()
        }

        Ok(())
    }
}
