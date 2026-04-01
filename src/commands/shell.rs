use crate::attribute_path::AttributePath;
use crate::error::NieResult;
use crate::interaction::inform_enter_shell;
use crate::location::NixReference;
use crate::{EvalArgs, nix};
use crate::store::output::NixOutput;


#[derive(clap::Args)]
pub struct ShellCommand {
    /// Nix references to fetch and add to shell
    #[arg(default_value = ".")]
    refs: Vec<NixReference>,

    /// Run COMMAND inside the shell
    #[arg(short, long)]
    command: Option<String>,

    #[clap(flatten)]
    eval_args: EvalArgs,

    /// Additional arguments for nix (see nix-shell(1))
    #[arg(last = true, allow_hyphen_values = true)]
    extra_args: Vec<String>,
}

impl super::Command for ShellCommand {
    fn exec(self) -> NieResult<()> {
        let default = AttributePath::common_package_locations();
        let paths: Vec<_> = NixOutput::fetch_and_build_all(&self.refs, &default, false, &self.eval_args, &[], None)?
            .into_iter()
            .flatten()
            .collect();

        inform_enter_shell(&paths);
        nix::shell(&paths, self.command, &self.eval_args, &self.extra_args)
    }
}
