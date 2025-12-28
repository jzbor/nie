use crate::error::NieResult;
use crate::location::NixReference;
use crate::store::output::NixOutput;


#[derive(clap::Args)]
pub struct BuildCommand {
    /// Nix references to fetch and build
    refs: Vec<NixReference>,

    /// Additional arguments for the nix builder (see nix-build(1))
    #[clap(last = true)]
    nix_args: Vec<String>,
}

impl super::Command for BuildCommand {
    fn exec(self) -> NieResult<()> {
        let paths: Vec<_> = NixOutput::fetch_and_build_all(&self.refs, true, &self.nix_args)?
            .into_iter()
            .flatten()
            .collect();

        paths.iter().for_each(|p| println!("{}", p.to_string_lossy()));
        Ok(())
    }
}
