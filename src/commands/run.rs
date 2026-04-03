use std::process;

use crate::error::{NieError, NieResult};
use crate::interact::{inform_open_man, inform_run_binary};
use crate::location::{AttributePath, NixReference};
use crate::store::NixFile;
use crate::{EvalArgs, nix};


#[derive(clap::Args)]
pub struct RunCommand {
    /// Nix references to fetch and add to shell
    #[arg(default_value = ".")]
    reference: NixReference,

    #[arg(short, long)]
    man: bool,

    #[clap(flatten)]
    eval_args: EvalArgs,

    /// Arguments passed to command
    #[arg(last = true)]
    args: Vec<String>,
}


impl super::Command for RunCommand {
    fn exec(self) -> NieResult<()> {
        let default = AttributePath::common_package_locations();
        let file = NixFile::fetch(self.reference.file(), self.eval_args)?;
        let output = file.output(self.reference.attribute().clone(), &default)?;
        output.build(None, false, &[], None)?;

        if self.man {
            let man_path = output.man_path()
                .ok_or_else(|| NieError::ManNotFound(self.reference.into()))?;

            inform_open_man(&man_path);
            match nix::exec("man", [man_path.to_string_lossy().to_string().as_str()]) {
                Err(NieError::ExternalCommand(_, code)) => process::exit(code),
                other => other,
            }
        } else {
            let bin_path = output.main_program()
                .ok_or_else(|| NieError::ProgramNotFound(self.reference.into()))?;

            inform_run_binary(&bin_path);
            println!();
            match nix::exec(bin_path.to_string_lossy().to_string().as_str(), self.args) {
                Err(NieError::ExternalCommand(_, code)) => process::exit(code),
                other => other,
            }
        }
    }
}
