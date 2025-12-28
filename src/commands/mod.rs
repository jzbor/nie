use crate::error::NieResult;

pub mod build;
pub mod completions;
pub mod dev_shell;
pub mod man;
pub mod run;
pub mod shell;
pub mod show;

pub trait Command: clap::Args {
    fn exec(self) -> NieResult<()>;
}
