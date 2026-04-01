use std::path::{Path, PathBuf};
use std::time::Duration;

use colored::Colorize;

use crate::attribute_path::AttributePath;
use crate::location::RepositoryReference;
use crate::store::file::NixFile;

pub fn inform(s: &str) {
    eprintln!("{}", s.to_string().bright_blue())
}

pub fn inform_fetch(repo: &RepositoryReference) {
    inform(&format!("Fetching {}", repo.to_string().underline()))
}

pub fn inform_build(attr: &AttributePath, file: &NixFile, flake_compat: bool, remote: Option<&str>) {
    inform(&format!("Building {} from ({}){}{}",
            attr.to_string_user().italic(),
            file.reference().to_string().underline(),
            if flake_compat { " with flake_compat" } else { "" },
            if let Some(remote) = remote { format!(" on {}", remote) } else { String::new() }));
}

pub fn inform_eval(attr: &AttributePath, file: &NixFile, flake_compat: bool) {
    inform(&format!("Building {} from ({}){}",
            attr.to_string_user().italic(),
            file.reference().to_string().underline(),
            if flake_compat { " with flake_compat" } else { "" }))
}

pub fn inform_create_gc_root(root: &Path, attr: &AttributePath, file: &NixFile) {
    inform(&format!("Creating gc root {} for derivation of {} ({})",
            root.to_string_lossy(),
            attr.to_string_user().italic(),
            file.reference().to_string().underline()));
}

pub fn inform_enter_shell(paths: &[PathBuf]) {
    inform(&format!("Entering shell with {} added paths", paths.len()));
}

pub fn inform_create_dev_shell(attr: &AttributePath, file: &NixFile) {
    inform(&format!("Entering dev shell {} from {}",
            attr.to_string_user().italic(),
            file.reference().to_string().underline()));
}

pub fn inform_create_dev_shell_pinned(link_age: Duration) {
    inform(&format!("Entering dev shell from local pin ({} days old)",
            link_age.as_secs() / (24 * 60 * 60)));
}

pub fn inform_run_binary(bin_path: &Path) {
    inform(&format!("Executing {}", bin_path.to_string_lossy()));
}

pub fn inform_open_man(man_path: &Path) {
    inform(&format!("Opening man page {}", man_path.to_string_lossy()));
}

pub fn inform_init_from_template(destination: &Path, source: &Path) {
    inform(&format!("Initializing {} from {}", destination.to_string_lossy(), source.to_string_lossy()));
}

pub fn inform_init_shell_nix(destination: &Path) {
    inform(&format!("Initializing {}", destination.to_string_lossy()));
}

pub fn announce(s: &str) {
    eprintln!("\n{}", format!("=> {}", s).bright_green())
}
