use std::{env, fs};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::SystemTime;

use crate::interaction::inform_create_dev_shell_pinned;
use crate::store::output::NixOutput;
use crate::{EvalArgs, nix};
use crate::attribute_path::AttributePath;
use crate::error::{NieError, NieResult};
use crate::store::file::NixFile;
use crate::location::NixReference;


const DEV_SHELL_DRV_ROOT: &str = ".nie-dev-shell/drv";
const DEV_SHELL_ROOT: &str = ".nie-dev-shell/path";


#[derive(clap::Args)]
pub struct DevelopCommand {
    /// Nix references to fetch and add to shell
    #[arg(default_value = ".")]
    reference: NixReference,

    /// Automatically enter development shell if local instantiation (see --pin) is found
    ///
    /// Do nothing otherwise.
    #[arg(long)]
    auto: bool,

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
            return unpin();
        }
        if self.auto {
            return auto(self);
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
            return pin(&output);
        }

        if reference.attribute().is_toplevel()
                && !self.no_pinned
                && !self.shell_nix
                && fs::exists(DEV_SHELL_DRV_ROOT)? {
            let link_age = SystemTime::elapsed(&fs::symlink_metadata(DEV_SHELL_DRV_ROOT)?.created()?)?;
            inform_create_dev_shell_pinned(link_age);
            nix::dev_shell(&PathBuf::from(DEV_SHELL_DRV_ROOT), &AttributePath::default(), &self.eval_args, command, &self.extra_args)
        } else {
            output.enter_dev_shell(command, &self.extra_args)
        }
    }
}

fn unpin() -> NieResult<()> {
    eprintln!("Removing {}", DEV_SHELL_DRV_ROOT);
    fs::remove_file(DEV_SHELL_DRV_ROOT)?;
    eprintln!("Removing {}", DEV_SHELL_ROOT);
    fs::remove_file(DEV_SHELL_ROOT)?;

    if let Some(parent) = PathBuf::from(DEV_SHELL_DRV_ROOT).parent()
            && parent.exists()
            && parent.read_dir()?.flatten().count() == 0 {
        eprintln!("Removing {}", parent.to_string_lossy());
        fs::remove_dir(parent)?;
    }

    if let Some(parent) = PathBuf::from(DEV_SHELL_ROOT).parent()
            && parent.exists()
            && parent.read_dir()?.flatten().count() == 0 {
        eprintln!("Removing {}", parent.to_string_lossy());
        fs::remove_dir(parent)?;
    }

    Ok(())
}

fn pin(output: &NixOutput) -> NieResult<()> {
    if fs::exists(DEV_SHELL_DRV_ROOT)? {
        fs::remove_file(DEV_SHELL_DRV_ROOT)?;
    }
    if fs::exists(DEV_SHELL_ROOT)? {
        fs::remove_file(DEV_SHELL_ROOT)?;
    }

    if let Some(parent) = PathBuf::from(DEV_SHELL_ROOT).parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = PathBuf::from(DEV_SHELL_DRV_ROOT).parent() {
        fs::create_dir_all(parent)?;
    }

    output.build(Some(DEV_SHELL_ROOT), true, &[], None)?;
    output.create_drv_gc_root(&PathBuf::from(DEV_SHELL_DRV_ROOT))?;

    Ok(())
}

fn auto(command: DevelopCommand) -> NieResult<()> {
    // Return if already in a Nix shell
    if env::var("IN_NIX_SHELL").is_ok() {
        return Ok(())
    }
    // Return if no local instantiation exists
    if !fs::exists(DEV_SHELL_DRV_ROOT)? {
        return Ok(())
    }
    let canon = fs::canonicalize(DEV_SHELL_DRV_ROOT)?;
    if !canon.starts_with("/nix/store/") {
        return Err(NieError::PinnedShellNotInStore(canon.to_string_lossy().to_string()))
    }

    let link_age = SystemTime::elapsed(&fs::symlink_metadata(DEV_SHELL_DRV_ROOT)?.created()?)?;
    inform_create_dev_shell_pinned(link_age);
    nix::dev_shell(&PathBuf::from(DEV_SHELL_DRV_ROOT), &AttributePath::default(), &command.eval_args, None, &command.extra_args)
}
