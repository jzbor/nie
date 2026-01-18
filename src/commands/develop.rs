use crate::EvalArgs;
use crate::attribute_path::AttributePath;
use crate::error::NieResult;
use crate::store::file::NixFile;
use crate::location::NixReference;


#[derive(clap::Args)]
pub struct DevelopCommand {
    /// Nix references to fetch and add to shell
    #[arg(default_value = "./.")]
    reference: NixReference,

    /// Run COMMAND inside the shell
    #[arg(short, long)]
    command: Option<String>,

    #[clap(flatten)]
    eval_args: EvalArgs,

    /// Additional arguments for nix (see nix-shell(1))
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    extra_args: Vec<String>,
}


impl super::Command for DevelopCommand {
    fn exec(self) -> NieResult<()> {
        let file = NixFile::fetch(self.reference.file(), self.eval_args)?;
        let output = file.output(self.reference.attribute().clone(), &AttributePath::common_dev_shell_locations())?;
        output.enter_dev_shell(self.command, &self.extra_args)
    }
}
