use std::fs::File;
use std::io::Write;
use std::time::{Duration, SystemTime};
use std::{env, fs, process};
use std::path::PathBuf;

use crate::error::{NieError, NieResult};
use crate::interact::{inform_enter_pinned_shell, inform_update_pinned_shell};
use crate::location::{AttributePath, NixReference};
use crate::{ENV_AUTOSHELL_DIR, ENV_AUTOSHELL_PID, EvalArgs, nix};
use crate::store::{NixFile, NixOutput};


const DEV_SHELL_DIR: &str = ".nie-dev-shell";
const DEV_SHELL_DRV_LINK: &str = "drv";
const DEV_SHELL_GCROOT_LINK: &str = "path";
const DEV_SHELL_REF_FILE: &str = "ref";


pub struct PinnedShell(PathBuf);

impl PinnedShell {
    /// Create new [`PinnedShell`] from a pin inside the current working directory
    pub fn new_from_cwd() -> NieResult<Self> {
        Self::new(env::current_dir()?)
    }

    /// Create new [`PinnedShell`] from a pin inside a certain directory
    pub fn new(path: PathBuf) -> NieResult<Self> {
        if !fs::exists(path.join(DEV_SHELL_DIR))? {
            Err(NieError::PinnedShellNotFound(path.to_string_lossy().to_string()))
        } else {
            Ok(PinnedShell(path))
        }
    }

    /// Create new [`PinnedShell`] by linking an output result to the current working directory
    pub fn create_at_cwd(output: &NixOutput) -> NieResult<Self> {
        let cwd = env::current_dir()?;
        Self::create(cwd, output)
    }

    /// Create new [`PinnedShell`] by linking an output result to a directory
    pub fn create(path: PathBuf, output: &NixOutput) -> NieResult<Self> {
        if fs::exists(path.join(DEV_SHELL_DIR))? {
            fs::remove_dir_all(DEV_SHELL_DIR)?;
        }

        fs::create_dir_all(path.join(DEV_SHELL_DIR))?;

        let gc_root_link = path.join(DEV_SHELL_DIR).join(DEV_SHELL_GCROOT_LINK);
        let drv_link = path.join(DEV_SHELL_DIR).join(DEV_SHELL_DRV_LINK);
        output.build(Some(&gc_root_link.to_string_lossy()), true, &[], None)?;
        output.create_drv_gc_root(&drv_link)?;

        let mut ref_file = File::create(path.join(DEV_SHELL_DIR).join(DEV_SHELL_REF_FILE))?;
        let ref_str = output.reference().to_string();
        ref_file.write_all(ref_str.as_bytes())?;

        Ok(PinnedShell(path))
    }

    /// Update the shell from it's saved reference ([`DEV_SHELL_REF_FILE`])
    pub fn update_from_ref(&mut self, eval_args: EvalArgs) -> NieResult<()> {
        let reference = self.reference()?;
        let file = NixFile::fetch(reference.file(), eval_args)?;
        let output = file.output(reference.attribute().clone(), &AttributePath::common_dev_shell_locations())?;
        self.update(&output)
    }

    /// Update the shell from a [`NixOutput`]
    pub fn update(&mut self, output: &NixOutput) -> NieResult<()> {
        inform_update_pinned_shell();
        let gc_root_link = self.gcroot_link();
        let drv_link = self.drv_link();
        output.build(Some(&gc_root_link.to_string_lossy()), true, &[], None)?;
        output.create_drv_gc_root(&drv_link)
    }

    /// Returns the project directory (parent of [`DEV_SHELL_DIR`])
    pub fn project_dir(&self) -> PathBuf {
        self.0.to_owned()
    }

    /// Returns the pin directory (fully qualified [`DEV_SHELL_DIR`])
    pub fn pin_dir(&self) -> PathBuf {
        self.0.join(DEV_SHELL_DIR)
    }

    /// Returns the link to the shell derivation ([`DEV_SHELL_DRV_LINK`])
    pub fn drv_link(&self) -> PathBuf {
        self.pin_dir().join(DEV_SHELL_DRV_LINK)
    }

    /// Returns the location of the link to the gc root ([`DEV_SHELL_GCROOT_LINK`])
    pub fn gcroot_link(&self) -> PathBuf {
        self.pin_dir().join(DEV_SHELL_GCROOT_LINK)
    }

    /// Returns the location of the link to the reference file for updates ([`DEV_SHELL_REF_FILE`])
    pub fn ref_file(&self) -> PathBuf {
        self.pin_dir().join(DEV_SHELL_REF_FILE)
    }

    /// Reads the stored reference ([`DEV_SHELL_REF_FILE`])
    pub fn reference(&self) -> NieResult<NixReference> {
        fs::read_to_string(self.ref_file())?
            .parse()
    }

    /// Reads the pin age as a [`Duration`]
    pub fn age(&self) -> NieResult<Duration> {
        let age = SystemTime::elapsed(&fs::symlink_metadata(self.drv_link())?.created()?)?;
        Ok(age)
    }

    /// Returns the location of the "reverse cd" link based on the parent process's id
    pub fn recd_link(&self) -> PathBuf {
        self.pin_dir().join(format!("tmp_recd_{}", std::os::unix::process::parent_id()))
    }

    /// Create an new "reverse cd" link based on [`ENV_AUTOSHELL_DIR`] and [`ENV_AUTOSHELL_PID`]
    pub fn create_recd_link() -> NieResult<()> {
        let orig_path = env::var(ENV_AUTOSHELL_DIR)
            .map_err(|_| NieError::NoReverseCd())?;
        let orig_pid: u32 = env::var(ENV_AUTOSHELL_PID)
            .map_err(|_| NieError::NoReverseCd())?
            .parse()
            .map_err(|_| NieError::NoReverseCd())?;

        let pinned = Self::new(orig_path.into())?;
        let recd_pin = pinned.pin_dir().join(format!("tmp_recd_{}", orig_pid));
        let cwd = env::current_dir()?;

        if fs::exists(&recd_pin)? {
            fs::remove_file(&recd_pin)?;
        }
        std::os::unix::fs::symlink(cwd, &recd_pin)?;

        Ok(())
    }

    /// [`true`] if the shell pin directory is checked into git.
    ///
    /// This is a sign for a potential security violation.
    pub fn is_git_registered(&self) -> bool {
        let pin_dir = self.pin_dir();
        process::Command::new("git")
            .args(["ls-files", "--error-unmatch", &pin_dir.to_string_lossy()])
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .status()
            .map(|e| e.success())
            .unwrap_or_default()
    }

    /// Check whether the pinned shell is safe, i.e. it has not been fetched through git and indeed
    /// points to a store path.
    pub fn is_safe(&self) -> NieResult<bool> {
        let canon = fs::canonicalize(self.drv_link())?;
        Ok(canon.starts_with("/nix/store/") && !self.is_git_registered())
    }

    /// Remove the pin
    pub fn remove(self) -> NieResult<()> {
        if fs::exists(self.pin_dir())? {
            fs::remove_dir_all(self.pin_dir())?;
        }

        Ok(())
    }

    /// Enter the pinned development shell
    pub fn enter(&self, command: Option<String>, eval_args: &EvalArgs, extra_args: &[String]) -> NieResult<()> {
        inform_enter_pinned_shell(self.age()?);
        nix::dev_shell(&self.drv_link(), &AttributePath::default(), eval_args, command, extra_args)
    }
}
