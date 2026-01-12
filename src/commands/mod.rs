use crate::error::NieResult;

pub mod aliases;
pub mod build;
pub mod completions;
pub mod develop;
pub mod man;
pub mod run;
pub mod shell;
pub mod show;

pub trait Command: clap::Args {
    fn exec(self) -> NieResult<()>;
}
