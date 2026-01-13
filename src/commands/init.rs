use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::BuildArgs;
use crate::attribute_path::AttributePath;
use crate::error::{NieError, NieResult};
use crate::interaction::announce;
use crate::location::NixReference;
use crate::store::checkout::Checkout;


#[derive(clap::Args)]
pub struct InitCommand {
    /// Nix references to fetch and build
    reference: NixReference,

    #[arg(default_value = ".")]
    destination: PathBuf,

    #[arg(short, long)]
    direct: bool,

    #[clap(flatten)]
    build_args: BuildArgs,
}

impl super::Command for InitCommand {
    fn exec(self) -> NieResult<()> {
        let common = AttributePath::common_template_locations();

        let checkout = Checkout::create(self.reference.repository().clone())?;
        let template = if self.direct {
            checkout.path().to_owned()
        } else {
            let file = checkout.file(self.reference.filename().cloned(), self.build_args.flake_compat)?;
            let mut output = file.output(self.reference.attribute().clone(), &common)?;

            if file.has_attribute(&output.attr().child("path".to_owned()))? {
                output = file.output(output.attr().child("path".to_owned()), &common)?;
            }

            output.eval(&self.build_args, &[])?
                .lines()
                .next()
                .map(|s| PathBuf::from(s))
                .ok_or_else(|| NieError::NoOutputPath(self.reference.into()))?
        };

        announce(&format!("Copying {} to {}", template.to_string_lossy(), self.destination.to_string_lossy()));
        copy(&template, &self.destination, true)
    }
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
