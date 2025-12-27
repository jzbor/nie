use crate::error::{NieError, NieResult};
use crate::file::NixFile;
use crate::interaction::announce;
use crate::location::NixReference;
use crate::nix;
use crate::output::NixOutput;


#[derive(clap::Args)]
pub struct RunCommand {
    /// Nix references to fetch and add to shell
    #[clap(default_value = "./.")]
    reference: NixReference,

    /// Additional arguments for the nix builder (see nix-build(1))
    #[clap(last = true)]
    args: Vec<String>,
}

impl super::Command for RunCommand {
    fn exec(self) -> NieResult<()> {
        let file = NixFile::fetch(self.reference.file())?;
        let output = file.output(self.reference.attribute().clone())?;
        let paths: Vec<_> = NixOutput::fetch_and_build(&self.reference, false, &[])?;
        let path = paths.first().ok_or(NieError::NoOutputPath(Box::new(self.reference)))?;
        let name = output.drv_name()?;
        let bin_path = path.join("bin").join(name);

        announce(&format!("Executing {}", bin_path.to_string_lossy()));
        nix::exec(bin_path.to_string_lossy().to_string().as_str(), self.args)
    }
}
