use std::fmt::Display;
use std::time::SystemTimeError;
use std::{io, process};

use colored::Colorize;

use crate::attribute_path::AttributePath;
use crate::location::NixReference;

pub type NieResult<T> = Result<T, NieError>;

#[derive(Debug, thiserror::Error)]
pub enum NieError {
    #[error("{0}")]
    Completions(String),

    #[error("{0}")]
    IO(#[from] io::Error),

    #[error("{0}")]
    Man(String),

    #[error("External command failed ({0})")]
    ExternalCommand(String, i32),

    #[error("Missing data from Nix evaluation ({0})")]
    MissingNixData(String),

    #[error("Failed to parse JSON ({0})")]
    Json(#[from] serde_json::Error),

    #[error("Failed to unfold JSON value ({0})")]
    JsonUnfolding(serde_json::Value),

    #[error("Broken output attribute ({0})")]
    BrokenAttribute(AttributePath),

    #[error("Could not find attribute \"{1}\" in file \"{0}\"")]
    AttributeNotFound(String, AttributePath),

    #[error("\"{0}\" should have been built, but does not exist (anymore)")]
    BuiltPathMissing(String),

    #[error("No output paths found for \"{0}\"")]
    NoOutputPath(Box<NixReference>),

    #[error("Invalid location specification \"{0}\"")]
    InvalidLocationSpec(String),

    #[error("Could not find file \"{0}\" in \"{1}\"")]
    NixFileNotFound(String, String),

    #[error("Could not find alias \"{0}\"")]
    AliasNotFound(String),

    #[error("Directory already exists: {0}")]
    DirectoryAlreadyExists(String),

    #[error("Program not found: {0}")]
    ProgramNotFound(Box<NixReference>),

    #[error("No checks found for {0}")]
    NoChecksFound(Box<NixReference>),

    #[error("Unable to calculate time: {0}")]
    SystemTime(#[from] SystemTimeError),

    #[error("Missing environment variable \"{0}\"")]
    EnvVarMissing(&'static str),

    #[error("Pinned shell points to unsafe path (\"{0}\")")]
    PinnedShellNotInStore(String),
}

pub fn resolve<T, E: Display>(result: Result<T, E>) -> T {
    match result {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            process::exit(1)
        },
    }
}
