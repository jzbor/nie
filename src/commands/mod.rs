use crate::error::NieResult;

pub mod build;
pub mod completions;
pub mod man;
pub mod shell;
pub mod show;
pub mod run;

pub trait Command: clap::Args {
    fn exec(self) -> NieResult<()>;
}
