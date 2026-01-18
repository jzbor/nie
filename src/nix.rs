use std::collections::{BTreeMap, VecDeque};
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{self, Stdio};

use crate::EvalArgs;
use crate::attribute_path::AttributePath;
use crate::error::{NieError, NieResult};


pub fn fetch_git(url: &str, args: &BTreeMap<String, String>) -> NieResult<PathBuf> {
    let out = exec_output("nix-instantiate", [
        "--eval",
        "--raw",
        "--expr",
        "--log-format", "bar",
        include_str!("./nix/fetch_git.nix"),
        "--arg", "url", &escape_url(url),
        "--arg", "args", serialize_args(args).as_str(),
    ])?;

    match out.lines().next() {
        Some(line) => Ok(PathBuf::from(line)),
        None => Err(NieError::MissingNixData(String::from("fetchGit store path"))),
    }
}

pub fn fetch_tarball(url: &str, args: &BTreeMap<String, String>) -> NieResult<PathBuf> {
    let out = exec_output("nix-instantiate", [
        "--eval",
        "--raw",
        "--expr",
        "--log-format", "bar",
        include_str!("./nix/fetch_tarball.nix"),
        "--arg", "url", &escape_url(url),
        "--arg", "args", serialize_args(args).as_str(),
    ])?;

    match out.lines().next() {
        Some(line) => Ok(PathBuf::from(line)),
        None => Err(NieError::MissingNixData(String::from("fetchGit store path"))),
    }
}

pub fn fetch_codeberg(owner: &str, repo: &str, gitref: Option<&str>, args: &BTreeMap<String, String>)
        -> NieResult<PathBuf> {
    let gitref_arg = if let Some(gitref) = gitref {
        format!("\"{}\"", gitref)
    } else {
        "null".to_owned()
    };

    let out = exec_output("nix-instantiate", [
        "--eval",
        "--raw",
        "--expr",
        "--log-format", "bar",
        include_str!("./nix/fetch_codeberg.nix"),
        "--arg", "owner", &format!("\"{}\"", owner),
        "--arg", "repo", &format!("\"{}\"", repo),
        "--arg", "ref", gitref_arg.as_str(),
        "--arg", "args", serialize_args(args).as_str(),
    ])?;

    match out.lines().next() {
        Some(line) => Ok(PathBuf::from(line)),
        None => Err(NieError::MissingNixData(String::from("fetchGit store path"))),
    }
}

pub fn fetch_github(owner: &str, repo: &str, branch: Option<&str>, args: &BTreeMap<String, String>)
        -> NieResult<PathBuf> {
    let branch_arg = if let Some(branch) = branch {
        format!("\"{}\"", branch)
    } else {
        "null".to_owned()
    };

    let out = exec_output("nix-instantiate", [
        "--eval",
        "--raw",
        "--expr",
        "--log-format", "bar",
        include_str!("./nix/fetch_github.nix"),
        "--arg", "owner", &format!("\"{}\"", owner),
        "--arg", "repo", &format!("\"{}\"", repo),
        "--arg", "branch", branch_arg.as_str(),
        "--arg", "args", serialize_args(args).as_str(),
    ])?;

    match out.lines().next() {
        Some(line) => Ok(PathBuf::from(line)),
        None => Err(NieError::MissingNixData(String::from("fetchGit store path"))),
    }
}

pub fn has_attribute(file: &Path, attr: &AttributePath, eval_args: &EvalArgs) -> NieResult<bool> {
    let output = if eval_args.flake_compat {
        let compat = include_str!("./nix/compat.nix");

        exec_output("nix-instantiate", [
            "--eval",
            file.join("flake.nix").to_string_lossy().to_string().as_str(),
            "--log-format", "bar",
        ])?;

        exec_output("nix-instantiate", [
            "--eval",
            "--expr",
            "--log-format", "bar",
            &format!("{{ path, }}: (({}) {{ inherit path; }}) ? {}", compat, attr),
            "--arg", "path", file.to_string_lossy().to_string().as_str(),
        ])?
    } else {
        exec_output("nix-instantiate", [
            "--eval",
            file.to_string_lossy().to_string().as_str(),
            "--log-format", "bar",
        ])?;

        exec_output("nix-instantiate", [
            "--eval",
            "--expr",
            "--log-format", "bar",
            &format!("{{ file, }}: (import file {}) ? {}", eval_args.expression_args_str()?, attr),
            "--arg", "file", file.to_string_lossy().to_string().as_str(),
        ])?
    };

    let found = output.trim()
        .parse()
        .unwrap_or_default();

    Ok(found)
}

pub fn is_lambda(file: &Path) -> NieResult<bool> {
    let output = exec_output("nix-instantiate", [
        "--eval",
        "--expr",
        "--raw",
        &format!("builtins.typeOf (import {})", file.to_string_lossy()),
        "--log-format", "bar",
    ])?;

    Ok(output.trim() == "lambda")
}

pub fn current_system() -> NieResult<String> {
    exec_output("nix-instantiate", [
        "--eval",
        "--raw",
        "-E",
        "builtins.currentSystem",
    ])
}

pub fn push_paths(paths: &[&Path], remote: &str) -> NieResult<()> {
    let mut args: VecDeque<_> =  paths.iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    args.push_front(remote.to_string());

    exec("nix-copy-closure", args)
}

pub fn pull_paths(paths: &[&Path], remote: &str) -> NieResult<()> {
    let mut args: VecDeque<_> =  paths.iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    args.push_front(remote.to_string());
    args.push_front("--from".to_string());

    exec("nix-copy-closure", args)
}

pub fn build_remote(inputs: &[&Path], path: &Path, attribute: &AttributePath, remote: &str, eval_args: &EvalArgs, extra_args: &[String])
        -> NieResult<Vec<PathBuf>> {
    push_paths(inputs, remote)?;

    let path_str = path.to_string_lossy().to_string();
    let mut args = vec![
        remote,
        "nix-build",
        "--log-format", "bar",
    ];

    if eval_args.flake_compat {
        args.push("--expr");
        args.push(include_str!("./nix/compat.nix"));

        args.push("--arg");
        args.push("path");
        args.push(path_str.as_str());
    } else {
        args.push(path_str.as_str());
    }

    let attribute_str = attribute.to_string();
    if !attribute.is_toplevel() {
        args.push("-A");
        args.push(&attribute_str);
    }

    args.push("--no-out-link");

    for (k, v) in eval_args.nix_options() {
        args.push("--option");
        args.push(k);
        args.push(v);
    }

    for (key, value) in eval_args.expression_args() {
        args.push("--arg");
        args.push(key);
        args.push(value);
    }

    args.extend(extra_args.iter().map(|s| s.as_str()));

    let out = exec_output("ssh", &args)?;
    let paths: Vec<_> = out.lines()
        .map(PathBuf::from)
        .collect();
    let path_refs: Vec<_> = paths.iter()
        .map(|p| p.as_path())
        .collect();

    pull_paths(&path_refs, remote)?;

    paths.into_iter()
        .map(|p| if p.exists() {
            Ok(p)
        } else {
            Err(NieError::BuiltPathMissing(p.to_string_lossy().into()))
        })
        .collect::<NieResult<_>>()
}

pub fn build(path: &Path, attribute: &AttributePath, allow_out_links: bool, eval_args: &EvalArgs, extra_args: &[String])
        -> NieResult<Vec<PathBuf>> {
    let path_str = path.to_string_lossy().to_string();
    let mut args = vec![
        "--log-format", "bar",
    ];

    if eval_args.flake_compat {
        args.push("--expr");
        args.push(include_str!("./nix/compat.nix"));

        args.push("--arg");
        args.push("path");
        args.push(path_str.as_str());
    } else {
        args.push(path_str.as_str());
    }

    let attribute_str = attribute.to_string();
    if !attribute.is_toplevel() {
        args.push("-A");
        args.push(&attribute_str);
    }

    if !allow_out_links {
        args.push("--no-out-link");
    }

    for (k, v) in eval_args.nix_options() {
        args.push("--option");
        args.push(k);
        args.push(v);
    }

    for (key, value) in eval_args.expression_args() {
        args.push("--arg");
        args.push(key);
        args.push(value);
    }

    args.extend(extra_args.iter().map(|s| s.as_str()));

    let out = exec_output("nix-build", &args)?;
    out.lines()
        .map(PathBuf::from)
        .map(|p| if p.exists() {
            Ok(p)
        } else {
            Err(NieError::BuiltPathMissing(p.to_string_lossy().into()))
        }).collect()
}

pub fn eval(path: &Path, attribute: &AttributePath, eval_args: &EvalArgs, extra_args: &[String]) -> NieResult<String> {
    let path_str = path.to_string_lossy().to_string();
    let mut args = vec!("--eval");

    if eval_args.flake_compat {
        args.push("--expr");
        args.push(include_str!("./nix/compat.nix"));

        args.push("--arg");
        args.push("path");
        args.push(path_str.as_str());
    } else {
        args.push(path_str.as_str());
    }

    let attribute_str = attribute.to_string();
    if !attribute.is_toplevel() {
        args.push("-A");
        args.push(&attribute_str);
    }

    for (key, value) in eval_args.expression_args() {
        args.push("--arg");
        args.push(key);
        args.push(value);
    }

    for (k, v) in eval_args.nix_options() {
        args.push("--option");
        args.push(k);
        args.push(v);
    }

    args.extend(extra_args.iter().map(|s| s.as_str()));

    exec_output("nix-instantiate", &args)
}

pub fn shell(paths: &[PathBuf], command: Option<String>, eval_args: &EvalArgs, extra_args: &[String]) -> NieResult<()> {
    let mut args = vec!();

    for path in paths {
        args.push("-p".to_owned());
        args.push(path.to_string_lossy().to_string());
    }

    for (k, v) in eval_args.nix_options() {
        args.push("--option".to_owned());
        args.push((*k).to_owned());
        args.push((*v).to_owned());
    }

    if let Some(cmd) = command {
        args.push("--command".to_owned());
        args.push(cmd);
    } else if let Ok(shell) = env::var("SHELL") {
        args.push("--command".to_owned());
        args.push(shell);
    }

    args.extend_from_slice(extra_args);
    exec("nix-shell", &args)
}

pub fn dev_shell(path: &Path, attribute: &AttributePath, eval_args: &EvalArgs, command: Option<String>, extra_args: &[String]) -> NieResult<()> {
    let path_str = path.to_string_lossy().to_string();
    let mut args = vec![
        "-A".to_string(),
        attribute.to_string(),
    ];

    if eval_args.flake_compat {
        args.push("--expr".to_owned());
        args.push(include_str!("./nix/compat.nix").to_owned());

        args.push("--arg".to_owned());
        args.push("path".to_owned());
        args.push(path_str.clone());
    } else {
        args.push(path_str);
    }

    for (key, value) in eval_args.expression_args() {
        args.push("--arg".to_owned());
        args.push(key.to_owned());
        args.push(value.to_owned());
    }

    for (k, v) in eval_args.nix_options() {
        args.push("--option".to_owned());
        args.push((*k).to_owned());
        args.push((*v).to_owned());
    }

    if let Some(cmd) = command {
        args.push("--command".to_owned());
        args.push(cmd);
    } else if let Ok(shell) = env::var("SHELL") {
        args.push("--command".to_owned());
        args.push(shell);
    }

    args.extend_from_slice(extra_args);
    exec("nix-shell", &args)
}

fn serialize_args(args: &BTreeMap<String, String>) -> String {
    let mut serialized_args = String::new();
    serialized_args.push_str("{ ");
    for (k, v) in args {
        serialized_args.push_str(k);
        serialized_args.push_str(" = ");
        serialized_args.push_str(v);
        serialized_args.push_str("; ");
    }
    serialized_args.push('}');
    serialized_args
}

fn escape_url(url: &str) -> String {
    if url == "." {
        "./.".to_owned()
    } else if url.starts_with("./") || url.starts_with("/") {
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
        let code = status.code().unwrap_or(1);
        Err(NieError::ExternalCommand(cmd.to_owned(), code))
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
        let code = output.status.code().unwrap_or(1);
        Err(NieError::ExternalCommand(cmd.to_owned(), code))
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
        let code = output.status.code().unwrap_or(1);
        Err(NieError::ExternalCommand(cmd.to_owned(), code))
    } else {
        let value = serde_json::from_str(String::from_utf8_lossy(&output.stdout).as_ref())?;
        Ok(value)
    }
}

