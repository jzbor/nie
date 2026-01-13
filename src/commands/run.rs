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
        let paths = output.build(false, &BuildArgs::default(), &[])?;
        let path = paths.first().ok_or(NieError::NoOutputPath(Box::new(self.reference.clone())))?;
        let main_program = file.output(output.attr().child("meta".to_owned()).child("mainProgram".to_owned()), &[]).ok();
        let bin_path = main_program
            .and_then(|mp| mp.eval(&BuildArgs::default(), &["--raw".to_string()]).ok())
            .map(|mp| path.join("bin").join(mp))
            .or_else(|| output.drv_name().ok().map(|n| path.join("bin").join(n)))
            .ok_or_else(|| NieError::ProgramNotFound(self.reference.into()))?;

        inform(&format!("Executing {}", bin_path.to_string_lossy()));
        println!();
        match nix::exec(bin_path.to_string_lossy().to_string().as_str(), self.args) {
            Err(NieError::ExternalCommand(_, code)) => process::exit(code),
            other => other,
        }
    }
}
