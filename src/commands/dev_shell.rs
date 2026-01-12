use crate::error::NieResult;
use crate::store::file::NixFile;
use crate::location::NixReference;


#[derive(clap::Args)]
pub struct DevShellCommand {
    /// Nix references to fetch and add to shell
    #[arg(default_value = "./.")]
    reference: NixReference,

    /// Additional arguments for the nix builder (see nix-build(1))
    #[arg(last = true)]
    extra_args: Vec<String>,
}

impl super::Command for DevShellCommand {
    fn exec(self) -> NieResult<()> {
        let file = NixFile::fetch(self.reference.file(), false)?;
        let output = file.output(self.reference.attribute().clone())?;
        output.enter_dev_shell(&self.extra_args)
    }
}
