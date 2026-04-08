use std::{env, fs};
use std::path::PathBuf;

use crate::error::{NieError, NieResult, warn};
use crate::pinning::PinnedShell;
use crate::{ENV_AUTOSHELL_DIR, ENV_AUTOSHELL_PID, EvalArgs};


#[derive(clap::Args)]
pub struct AutoShellCommand {
    /// Try to update the shell before entering
    #[arg(long)]
    update: bool,

    /// Emit `exit` command if the current directory is outside of the projects directory
    ///
    /// This has to be placed **after** the normal `nie auto-shell` hook inside an `eval` argument.
    /// For example:
    /// ```sh
    /// nie auto-shell
    /// eval "$(nie auto-shell --auto-exit)"
    /// ```
    ///
    /// ***Note:** This option is somewhat fragile and it's implementation as well as it's presence
    /// itself may be reevaluated and changed in the future.*
    #[arg(long)]
    auto_exit: bool,

    #[clap(flatten)]
    eval_args: EvalArgs,

    /// Additional arguments for nix (see nix-shell(1))
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    extra_args: Vec<String>,
}


impl super::Command for AutoShellCommand {
    fn exec(self) -> NieResult<()> {

        if self.auto_exit {
            return auto_exit();
        }

        // Return if already in a Nix shell
        if in_shell() {
            return Ok(())
        }

        // Return if no local instantiation exists
        let mut pinned_shell = match PinnedShell::new_from_cwd() {
            Ok(shell) => shell,
            Err(_) => return Ok(()),
        };

        if !pinned_shell.is_safe()? {
            return Err(NieError::PinnedShellNotSafe(pinned_shell.pin_dir().to_string_lossy().to_string()))
        }

        if self.update {
            let _ = pinned_shell.update_from_ref(self.eval_args.clone())
                .map_err(warn);
        }

        unsafe {
            // env::set_var() is safe as long as no other thread is active.
            // This should be the case here.
            // If this changes at any point this unsafe block MUST be replaced.
            env::set_var(ENV_AUTOSHELL_DIR, pinned_shell.project_dir().to_string_lossy().to_string());
            env::set_var(ENV_AUTOSHELL_PID, std::os::unix::process::parent_id().to_string());
        }
        pinned_shell.enter(None, &self.eval_args, &self.extra_args)
    }
}

fn in_shell() -> bool {
    env::var("IN_NIX_SHELL").is_ok()
}

fn auto_exit() -> NieResult<()> {
    if let Ok(auto_dir) = env::var(ENV_AUTOSHELL_DIR) {
        let cwd = env::current_dir()?;
        let auto_dir = PathBuf::from(auto_dir);
        if !cwd.starts_with(auto_dir) {
            PinnedShell::create_recd_link()?;
            println!("exit");
        }
    } else if !in_shell() && let Ok(pinned) = PinnedShell::new_from_cwd() {
        let link = pinned.recd_link();
        if fs::exists(&link)? {
            println!("if [ -L \"{0}\" ]; then cd \"{1}\"; unlink \"{0}\"; fi",
                link.to_string_lossy(), link.canonicalize()?.to_string_lossy());
        }
    }
    Ok(())
}
