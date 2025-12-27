use crate::error::NieResult;
use crate::interaction::announce;
use crate::location::NixReference;
use crate::nix;
use crate::output::NixOutput;


#[derive(clap::Args)]
pub struct ShellCommand {
    /// Nix references to fetch and add to shell
    #[clap(default_value = "./.")]
    refs: Vec<NixReference>,

    /// Additional arguments for the nix builder (see nix-build(1))
    #[clap(last = true)]
    nix_args: Vec<String>,
}

impl super::Command for ShellCommand {
    fn exec(self) -> NieResult<()> {
        let paths: Vec<_> = NixOutput::fetch_and_build_all(&self.refs, false, &self.nix_args)?
            .into_iter()
            .flatten()
            .collect();

        announce(&format!("Entering shell with {} added paths", paths.len()));
        nix::shell(&paths, &self.nix_args)
    }
}
