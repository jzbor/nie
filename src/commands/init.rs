use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::error::{NieError, NieResult};
use crate::interact::{inform_init_from_template, inform_init_shell_nix};
use crate::location::{AttributePath, NixReference};
use crate::store::Checkout;
use crate::EvalArgs;


#[derive(clap::Args)]
pub struct InitCommand {
    #[clap(flatten)]
    template_type: TemplateType,

    /// Target directory to initialize
    #[arg(default_value = ".")]
    destination: PathBuf,

    /// Use reference repository directly as template
    #[arg(long)]
    repo: bool,

    #[clap(flatten)]
    eval_args: EvalArgs,
}

#[derive(clap::Args, Clone)]
#[group(required = true, multiple = false)]
struct TemplateType {
    /// Nix reference to use as a template
    reference: Option<NixReference>,

    /// Create a shell.nix file.
    #[arg(short, long)]
    shell_nix: bool,

    /// Create a default.nix file.
    #[arg(short, long)]
    default_nix: bool,
}


impl super::Command for InitCommand {
    fn exec(self) -> NieResult<()> {
        if let Some(reference) = self.template_type.reference {
            init_by_reference(&reference, &self.destination, self.eval_args, self.repo)
        } else if self.template_type.default_nix {
            init_default_nix(&self.destination)
        } else if self.template_type.shell_nix {
            init_shell_nix(&self.destination)
        } else {
            panic!("Invalid template type");
        }
    }
}

fn init_shell_nix(parent: &Path) -> NieResult<()> {
    let content = include_str!("../nix/template-shell.nix");
    let path = parent.join("shell.nix");
    inform_init_shell_nix(&path);
    fs::write(&path, content.as_bytes())?;
    Ok(())
}

fn init_default_nix(parent: &Path) -> NieResult<()> {
    let content = include_str!("../nix/template-default.nix");
    let path = parent.join("default.nix");
    inform_init_shell_nix(&path);
    fs::write(&path, content.as_bytes())?;
    Ok(())
}

fn init_by_reference(reference: &NixReference, destination: &Path, eval_args: EvalArgs, direct: bool) -> NieResult<()> {
    let common = AttributePath::common_template_locations();
    let checkout = Checkout::fetch(reference.repository().clone())?;
    let template = if direct {
        checkout.path().to_owned()
    } else {
        let file = checkout.file(reference.filename().cloned(), eval_args)?;
        let mut output = file.output(reference.attribute().clone(), &common)?;

        if file.has_attribute(&output.attr().child("path".to_owned()))? {
            output = file.output(output.attr().child("path".to_owned()), &common)?;
        }

        output.eval(&[])?
            .lines()
            .next()
            .map(PathBuf::from)
            .ok_or_else(|| NieError::NoOutputPath(reference.clone().into()))?
    };

    inform_init_from_template(destination, &template);
    copy(&template, destination, true)
}

fn copy(from: &Path, to: &Path, toplevel: bool) -> NieResult<()> {
    if from.metadata()?.is_dir() {
        if !fs::exists(to)? {
            fs::create_dir(to)?;
        } else if !toplevel || fs::read_dir(to)?.count() != 0 {
            return Err(NieError::DirectoryAlreadyExists(to.to_string_lossy().to_string()));
        }

        let from_perms = from.metadata()?.permissions();
        let perms = Permissions::from_mode((from_perms.mode() | 0o600) & 0o700);
        fs::set_permissions(to, perms)?;

        for entry in fs::read_dir(from)? {
            let entry = entry?;
            let child_from = from.join(entry.file_name());
            let child_to = to.join(entry.file_name());
            copy(&child_from, &child_to, false)?;
        }
    } else {
        fs::copy(from, to)?;

        let from_perms = from.metadata()?.permissions();
        let perms = Permissions::from_mode((from_perms.mode() | 0o600) & 0o700);
        fs::set_permissions(to, perms)?;
    }

    Ok(())
}
