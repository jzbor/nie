use std::iter;

use crate::store::checkout::Checkout;
use crate::error::NieResult;
use crate::location::NixReference;


#[derive(clap::Args)]
pub struct ShowCommand {
    /// Nix references to fetch and show
    #[arg(default_value = "./.")]
    refs: Vec<NixReference>,

    #[arg(short, long, default_value_t = 5)]
    depth: u32,

    #[arg(short, long)]
    reject_broken: bool,

    #[arg(short, long)]
    flake_compat: bool,
}

impl super::Command for ShowCommand {
    fn exec(self) -> NieResult<()> {
        let repo_refs = self.refs.iter().map(|s| s.repository()).cloned();
        let filenames = self.refs.iter().map(|s| s.filename().cloned());
        let checkouts = Checkout::create_all(repo_refs)?;
        let files = Checkout::files(iter::zip(checkouts.iter().cloned(), filenames), self.flake_compat)?;

        println!();

        for (reference, file) in iter::zip(self.refs.into_iter(), files.into_iter()) {
            println!("[{}]:", reference);
            file.attributes(self.depth, self.reject_broken)?
                .filter(|a| !(file.flake_compat() && a.len() > 1 && a.first().map(|c| c == "outputs").unwrap_or_default()))
                .for_each(|a| println!("{:>width$}{}", "", a.name().unwrap_or_default(), width=(a.depth() + 1) * 2));
            println!()
        }

        Ok(())
    }
}
