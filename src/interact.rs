//! Implementation of user interaction/output messages
use std::path::{Path, PathBuf};
use std::time::Duration;

use colored::Colorize;

use crate::error::NieError;
use crate::location::AttributePath;
use crate::location::RepositoryReference;
use crate::store::{NixFile, NixOutput};


/// Format [`Duration`] into a human-readable string
pub fn fmt_duration(duration: Duration) -> String {
    let hours = duration.as_secs() / 3600;
    let minutes = duration.as_secs() / 60 % 60;
    let seconds = duration.as_secs() % 60;
    let millis = duration.subsec_millis();
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s {}ms", minutes, seconds, millis)
    } else if seconds > 0 {
        format!("{}s {}ms", seconds, millis)
    } else {
        format!("{}ms", millis)
    }
}

/// Inform the user about fetching multiple sources (see [`inform()`])
pub fn inform_fetch_multiple(repos: &[RepositoryReference]) {
    let listed = match repos {
        [] => String::new(),
        [r0] => r0.to_string().underline().to_string(),
        [r0, r1] => format!("{} and {}", r0.to_string().underline(), r1.to_string().underline()),
        [r0, r1, r2] => format!("{}, {} and {}", r0.to_string().underline(),
                                r1.to_string().underline(), r2.to_string().underline()),
        [r0, r1, _, ..] => format!("{}, {} and {} more", r0.to_string().underline(),
                                   r1.to_string().underline(), repos.len() - 2),
    };
    inform(&format!("Fetching {}", listed))
}

/// Inform the user about building an output (see [`inform()`])
pub fn inform_build(attr: &AttributePath, file: &NixFile, flake_compat: bool, remote: Option<&str>) {
    inform(&format!("Building {} from ({}){}{}",
            attr.to_string_user().italic(),
            file.reference().to_string().underline(),
            if flake_compat { " with flake_compat" } else { "" },
            if let Some(remote) = remote { format!(" on {}", remote) } else { String::new() }));
}

/// Inform the user about evaluating an output (see [`inform()`])
pub fn inform_eval(attr: &AttributePath, file: &NixFile, flake_compat: bool) {
    inform(&format!("Building {} from ({}){}",
            attr.to_string_user().italic(),
            file.reference().to_string().underline(),
            if flake_compat { " with flake_compat" } else { "" }))
}

/// Inform the user about creating a gc root for an output (see [`inform()`])
pub fn inform_create_gc_root(root: &Path, attr: &AttributePath, file: &NixFile) {
    inform(&format!("Creating gc root {} for derivation of {} ({})",
            root.to_string_lossy(),
            attr.to_string_user().italic(),
            file.reference().to_string().underline()));
}

/// Inform the user about entering a shell (see [`inform()`])
pub fn inform_enter_shell(paths: &[PathBuf]) {
    inform(&format!("Entering shell with {} added paths", paths.len()));
}

/// Inform the user about entering a development shell (see [`inform()`])
pub fn inform_enter_dev_shell(attr: &AttributePath, file: &NixFile) {
    inform(&format!("Entering dev shell {} from {}",
            attr.to_string_user().italic(),
            file.reference().to_string().underline()));
}

/// Inform the user about entering a previously pinned development shell (see [`inform()`])
pub fn inform_enter_pinned_shell(link_age: Duration) {
    inform(&format!("Entering dev shell from local pin ({} days old)",
            link_age.as_secs() / (24 * 60 * 60)));
}

/// Inform the user about entering a previously pinned development shell (see [`inform()`])
pub fn inform_update_pinned_shell() {
    inform("Updating local dev shell pin");
}

/// Inform the user about running a binary (see [`inform()`])
pub fn inform_run_binary(bin_path: &Path) {
    inform(&format!("Executing {}", bin_path.to_string_lossy()));
}

/// Inform the user about opening a man page (see [`inform()`])
pub fn inform_open_man(man_path: &Path) {
    inform(&format!("Opening man page {}", man_path.to_string_lossy()));
}

/// Inform the user about initializing a directory from a template (see [`inform()`])
pub fn inform_init_from_template(destination: &Path, source: &Path) {
    inform(&format!("Initializing {} from {}", destination.to_string_lossy(), source.to_string_lossy()));
}

/// Inform the user about initializing a shell.nix file (see [`inform()`])
pub fn inform_init_shell_nix(destination: &Path) {
    inform(&format!("Initializing {}", destination.to_string_lossy()));
}

/// Inform the user about the checks that are about to be run
pub fn inform_running_checks(checks: &[NixOutput]) {
        println!();
        announce(&format!("Running {} checks:", checks.len()));
        for check in checks {
            println!("  🔳 {}", check.reference().attribute());
        }
        println!();
}

/// Inform the user about the completion of a check
pub fn inform_check_done(attr: &AttributePath, success: bool, duration: Duration) {
    let status = if success {
        "SUCCESSFULLY".green().bold()
    } else {
        "FAILED".red().bold()
    };
    println!("{} to build \"{}\" (took {})", status, attr, fmt_duration(duration))
}

/// Inform the user about the results of the checks that were performed
pub fn inform_check_summary(checks: &[NixOutput], results: &[Result<Vec<PathBuf>, NieError>], duration: Duration) {
    println!();
    announce(&format!("Results (took {}):", fmt_duration(duration)));
    for (i, check) in checks.iter().enumerate() {
        let result = results.get(i);
        match result {
            Some(Ok(_)) => println!("  ✅ {}", check.reference().attribute()),
            Some(Err(_)) => println!("  ❌ {}", check.reference().attribute()),
            None => println!("  ❔ {}", check.reference().attribute()),
        }
    }
    println!();
    if results.iter().any(Result::is_err) {
        println!("    {}", "FAILURE".red().bold())
    } else {
        println!("    {}", "SUCCESS".green().bold())
    }
    println!();

}

/// Inform the user about something (highlights the message)
pub fn inform(s: &str) {
    eprintln!("{}", s.to_string().bright_blue())
}

/// Announce an action or topic
pub fn announce(s: &str) {
    eprintln!("\n{}", format!("=> {}", s).bright_green())
}
