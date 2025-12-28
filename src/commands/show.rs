use std::iter;

use crate::store::checkout::Checkout;
use crate::error::NieResult;
use crate::location::NixReference;


#[derive(clap::Args)]
pub struct ShowCommand {
    /// Nix references to fetch and show
    #[clap(default_value = "./.")]
    refs: Vec<NixReference>,

    #[clap(short, long, default_value_t = 5)]
    depth: u32,

    #[clap(short, long)]
    reject_broken: bool,
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
            file.attributes(self.depth, self.reject_broken)?
                .for_each(|a| println!("{:>width$}{}", "", a.name().unwrap_or_default(), width=(a.depth() + 1) * 2));
            println!()
        }

        Ok(())
    }
}
