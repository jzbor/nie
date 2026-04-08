use std::str::FromStr;

use crate::error::{NieError, NieResult, warn};
use crate::location::{AttributePath, NixReference};
use crate::pinning::PinnedShell;
use crate::store::NixFile;
use crate::EvalArgs;


#[derive(clap::Args)]
pub struct DevelopCommand {
    /// Nix references to fetch and add to shell
    #[arg(default_value = ".")]
    reference: NixReference,

    /// Run COMMAND inside the shell
    #[arg(short, long)]
    command: Option<String>,

    /// Run $EDITOR inside the shell
    #[arg(short, long)]
    editor: bool,

    /// Use local shell.nix as source
    #[arg(short, long)]
    shell_nix: bool,

    /// Create a garbage collection root for the devShell and exit
    #[arg(short, long)]
    pin: bool,

    /// Like --pin, but does nothing if there is not already a pin
    #[arg(long)]
    update_pin: bool,

    /// Do not use pinned dev shells if found
    #[arg(short, long)]
    no_pinned: bool,

    /// Remove devShell gc roots
    #[arg(short, long)]
    unpin: bool,

    #[clap(flatten)]
    eval_args: EvalArgs,

    /// Additional arguments for nix (see nix-shell(1))
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    extra_args: Vec<String>,
}


impl super::Command for DevelopCommand {
    fn exec(self) -> NieResult<()> {
        if self.unpin {
            return PinnedShell::new_from_cwd()?.remove();
        }
        if self.update_pin {
            return PinnedShell::new_from_cwd()?.update_from_ref(self.eval_args);
        }

        let reference = if self.shell_nix {
            NixReference::from_str("file://.#file=shell.nix#nocopy=true")?
        } else {
            self.reference
        };

        let file = NixFile::fetch(reference.file(), self.eval_args.clone())?;
        let output = file.output(reference.attribute().clone(), &AttributePath::common_dev_shell_locations())?;
        let command = if self.editor { Some(String::from("$EDITOR")) } else { self.command };

        if self.pin {
            return PinnedShell::create_at_cwd(&output).map(|_| ());
        }


        if reference.attribute().is_toplevel()
                && !self.no_pinned
                && !self.shell_nix
                && let Ok(pinned_shell) = PinnedShell::new_from_cwd() {
            if !pinned_shell.is_safe()? {
                warn(NieError::PinnedShellNotSafe(pinned_shell.pin_dir().to_string_lossy().to_string()));
            }
            pinned_shell.enter(command, &self.eval_args, &self.extra_args)
        } else {
            output.enter_dev_shell(command, &self.extra_args)
        }
    }
}
