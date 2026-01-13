use crate::attribute_path::AttributePath;
use crate::error::NieResult;
use crate::interaction::announce;
use crate::location::NixReference;
use crate::{BuildArgs, nix};
use crate::store::output::NixOutput;


#[derive(clap::Args)]
pub struct ShellCommand {
    /// Nix references to fetch and add to shell
    #[arg(default_value = "./.")]
    refs: Vec<NixReference>,

    #[clap(flatten)]
    build_args: BuildArgs,

    #[arg(short, long)]
    command: Option<String>,
}

impl super::Command for ShellCommand {
    fn exec(self) -> NieResult<()> {
        let default = AttributePath::common_package_locations();
        let paths: Vec<_> = NixOutput::fetch_and_build_all(&self.refs, &default, false, &self.build_args, &[])?
            .into_iter()
            .flatten()
            .collect();

        announce(&format!("Entering shell with {} added paths", paths.len()));
        nix::shell(&paths, self.command, &self.build_args.nix_options())
    }
}
