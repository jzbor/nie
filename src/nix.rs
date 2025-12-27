use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{self, Stdio};

use crate::error::{NieError, NieResult};
use crate::location::AttributePath;

pub fn fetch_git(url: &str, args: &BTreeMap<String, String>) -> NieResult<PathBuf> {
    let mut serialized_args = String::new();
    serialized_args.push_str("{ ");
    for (k, v) in args {
        serialized_args.push_str(k);
        serialized_args.push_str(" = ");
        serialized_args.push('"');
        serialized_args.push_str(v);
        serialized_args.push('"');
        serialized_args.push_str("; ");
    }
    serialized_args.push('}');

    let out = exec_output("nix-instantiate", [
        "--eval",
        "--raw",
        "--expr",
        "--log-format", "bar",
        "{ url, args }: fetchGit ({ inherit url; submodules = true; shallow = true; } // args)",
        "--arg", "url", &escape_url(url),
        "--arg", "args", serialized_args.as_str(),
    ])?;

    match out.lines().next() {
        Some(line) => Ok(PathBuf::from(line)),
        None => Err(NieError::MissingNixData(String::from("fetchGit store path"))),
    }
}

pub fn has_attribute(file: &Path, attr: &AttributePath) -> NieResult<bool> {
    exec_output("nix-instantiate", [
        "--eval",
         file.to_string_lossy().to_string().as_str(),
        "--log-format", "bar",
    ])?;

    let found = exec_quiet("nix-instantiate", [
        "--eval",
        "--raw",
        "--expr",
        "--log-format", "bar",
        &format!("{{ file, }}: (import file {{}}).{}", attr),
        "--arg", "file", file.to_string_lossy().to_string().as_str(),
    ]).is_ok();

    Ok(found)
}

pub fn build(path: &Path, attribute: &AttributePath, out_link: bool, extra_args: &[String]) -> NieResult<Vec<PathBuf>> {
    let mut args = vec![
        path.to_string_lossy().to_string(),
    ];

    if !attribute.is_toplevel() {
        args.push("-A".to_owned());
        args.push(attribute.to_string());
    }

    if !out_link {
        args.push("--no-out-link".to_owned());
    }

    args.extend_from_slice(extra_args);
    let out = exec_output("nix-build", &args)?;
    out.lines()
        .map(PathBuf::from)
        .map(|p| if p.exists() {
            Ok(p)
        } else {
            Err(NieError::BuiltPathMissing(p.to_string_lossy().into()))
        }).collect()
}

pub fn shell(paths: &[PathBuf], extra_args: &[String]) -> NieResult<()> {
    let mut args = vec!();

    for path in paths {
        args.push("-p".to_owned());
        args.push(path.to_string_lossy().to_string());
    }

    if let Ok(shell) = env::var("SHELL") {
        args.push("--command".to_owned());
        args.push(shell);
    }

    args.extend_from_slice(extra_args);
    exec("nix-shell", &args)
}

fn escape_url(url: &str) -> String {
    if url.starts_with("./") || url.starts_with("/") {
        url.to_owned()
    } else {
        format!("\"{}\"", url)
    }
}

pub fn exec(cmd: &str, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> NieResult<()> {
    let status = process::Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        Err(NieError::ExternalCommand(cmd.to_owned()))
    } else {
        Ok(())
    }
}

pub fn exec_quiet(cmd: &str, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> NieResult<()> {
    let status = process::Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        Err(NieError::ExternalCommand(cmd.to_owned()))
    } else {
        Ok(())
    }
}

pub fn exec_output(cmd: &str, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> NieResult<String> {
    let output = process::Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    if !output.status.success() {
        Err(NieError::ExternalCommand(cmd.to_owned()))
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).into())
    }
}

pub fn exec_output_json(cmd: &str, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> NieResult<serde_json::Value> {
    let output = process::Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    if !output.status.success() {
        Err(NieError::ExternalCommand(cmd.to_owned()))
    } else {
        let value = serde_json::from_str(String::from_utf8_lossy(&output.stdout).as_ref())?;
        Ok(value)
    }
}

