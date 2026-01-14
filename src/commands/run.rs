use std::process;

use crate::attribute_path::AttributePath;
use crate::error::{NieError, NieResult};
use crate::interaction::inform;
use crate::location::NixReference;
use crate::store::file::NixFile;
use crate::{BuildArgs, nix};


#[derive(clap::Args)]
pub struct RunCommand {
    /// Nix references to fetch and add to shell
    #[arg(default_value = "./.")]
    reference: NixReference,

    /// Arguments passed to command
    #[arg(last = true)]
    args: Vec<String>,
}


impl super::Command for RunCommand {
    fn exec(self) -> NieResult<()> {
        let default = AttributePath::common_package_locations();
        let file = NixFile::fetch(self.reference.file(), false)?;
        let output = file.output(self.reference.attribute().clone(), &default)?;
        output.build(false, &BuildArgs::default(), &[])?;
        let bin_path = output.main_program()
            .ok_or_else(|| NieError::ProgramNotFound(self.reference.into()))?;

        inform(&format!("Executing {}", bin_path.to_string_lossy()));
        println!();
        match nix::exec(bin_path.to_string_lossy().to_string().as_str(), self.args) {
            Err(NieError::ExternalCommand(_, code)) => process::exit(code),
            other => other,
        }
    }
}
