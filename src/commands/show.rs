use std::iter;

use crate::interaction::announce;
use crate::store::checkout::Checkout;
use crate::error::NieResult;
use crate::location::NixReference;


#[derive(clap::Args)]
pub struct ShowCommand {
    /// Nix references to fetch and show
    #[arg(default_value = "./.")]
    refs: Vec<NixReference>,

    /// Maximal depth to search for
    #[arg(short, long, default_value_t = 5)]
    depth: u32,

    /// Print only the names of attributes, not their full path
    #[arg(short, long)]
    short: bool,

    /// Error out on broken packages
    #[arg(short, long)]
    reject_broken: bool,

    /// Force flake compatibility
    #[arg(short, long)]
    flake_compat: bool,
}

impl super::Command for ShowCommand {
    fn exec(self) -> NieResult<()> {
        let repo_refs = self.refs.iter().map(|s| s.repository()).cloned();
        let filenames = self.refs.iter().map(|s| s.filename().cloned());
        let checkouts = Checkout::create_all(repo_refs)?;
        let files = Checkout::files(iter::zip(checkouts.iter().cloned(), filenames), self.flake_compat)?;

        for (reference, file) in iter::zip(self.refs.into_iter(), files.into_iter()) {
            let attributes = file.attributes(self.depth, self.reject_broken)?;
            announce(&format!("Outputs in \"{}\":", reference));
            for attr in attributes {
                if file.flake_compat() && attr.len() > 1 && attr.first().map(|c| c == "outputs").unwrap_or_default() {
                    continue;
                }

                if self.short {
                    println!("{:>width$}{}", "", attr.name().unwrap_or_default(), width=(attr.depth()) * 2)
                } else {
                    println!("{:>width$}{}", "", attr, width=(attr.depth()) * 2)
                }
            }
        }

        println!();

        Ok(())
    }
}
