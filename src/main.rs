use clap::Parser;

use crate::commands::Command;


mod commands;
mod error;
mod interaction;
mod location;
mod store;
mod nix;
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

    /// Build a package from a Nix repo
    Build(commands::build::BuildCommand),

    /// Enter a dev shell from a Nix repo
    #[clap(alias = "develop")]
    DevShell(commands::dev_shell::DevShellCommand),

    /// Generate man pages
    #[clap(hide = true)]
    Man(commands::man::ManCommand),

    /// Run an executable from a Nix repo
    Run(commands::run::RunCommand),

    /// Enter a shell containing a package from a Nix repo
    Shell(commands::shell::ShellCommand),

    /// Show outputs of a package
    Show(commands::show::ShowCommand),
}

fn main() {
    let args = Args::parse();

    use Subcommand::*;
    let result = match args.subcommand {
        Build(cmd) => cmd.exec(),
        Completions(cmd) => cmd.exec(),
        DevShell(cmd) => cmd.exec(),
        Man(cmd) => cmd.exec(),
        Run(cmd) => cmd.exec(),
        Shell(cmd) => cmd.exec(),
        Show(cmd) => cmd.exec(),
    };

    error::resolve(result)
}
