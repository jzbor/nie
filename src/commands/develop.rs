use crate::attribute_path::AttributePath;
use crate::error::NieResult;
use crate::store::file::NixFile;
use crate::location::NixReference;


#[derive(clap::Args)]
pub struct DevelopCommand {
    /// Nix references to fetch and add to shell
    #[arg(default_value = "./.")]
    reference: NixReference,

    /// Additional arguments for the nix builder (see nix-build(1))
    #[arg(last = true)]
    extra_args: Vec<String>,
}

impl super::Command for DevelopCommand {
    fn exec(self) -> NieResult<()> {
        let file = NixFile::fetch(self.reference.file(), false)?;
        let output = file.output(self.reference.attribute().clone(), &AttributePath::default_dev_shells())?;
        output.enter_dev_shell(&self.extra_args)
    }
}
