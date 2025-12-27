use clap::Parser;

use crate::commands::Command;


mod checkout;
mod commands;
mod error;
mod file;
mod interaction;
mod location;
mod nix;
mod output;
mod registry;


#[derive(Parser)]
#[command(version, about, long_about)]
pub struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    /// Generate shell completions
    #[clap(hide = true)]
    Completions(commands::completions::CompletionsCommand),

    /// Generate man pages
    #[clap(hide = true)]
    Man(commands::man::ManCommand),

    /// Build a package from a Nix repo
    Build(commands::build::BuildCommand),

    /// Enter a shell containing a package from a Nix repo
    Shell(commands::shell::ShellCommand),

    /// Show outputs of a package
    Show(commands::show::ShowCommand),
}

fn main() {
    let args = Args::parse();

    use Subcommand::*;
    let result = match args.subcommand {
        Completions(cmd) => cmd.exec(),
        Man(cmd) => cmd.exec(),
        Build(cmd) => cmd.exec(),
        Show(cmd) => cmd.exec(),
        Shell(cmd) => cmd.exec(),
    };

    error::resolve(result)
}
