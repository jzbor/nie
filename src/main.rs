use clap::Parser;

use crate::commands::Command;


mod attribute_path;
mod commands;
mod error;
mod interaction;
mod location;
mod store;
mod nix;
mod registry;
mod aliases;


#[derive(Parser)]
#[command(version, about, long_about)]
pub struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    /// Show and update aliases
    Aliases(commands::aliases::AliasesCommand),

    /// Generate shell completions
    #[command(hide = true)]
    Completions(commands::completions::CompletionsCommand),

    /// Build a package from a Nix repo
    Build(commands::build::BuildCommand),

    /// Enter a dev shell from a Nix repo
    #[command(alias = "dev", alias = "dev-shell")]
    Develop(commands::develop::DevelopCommand),

    /// Generate man pages
    #[command(hide = true)]
    Man(commands::man::ManCommand),

    /// Run an executable from a Nix repo
    Run(commands::run::RunCommand),

    /// Enter a shell containing a package from a Nix repo
    Shell(commands::shell::ShellCommand),

    /// Show outputs of a package
    Show(commands::show::ShowCommand),
}

#[derive(Debug, Default, clap::Args)]
struct BuildArgs {
    /// Enable Flake compatibility
    #[arg(short, long)]
    flake_compat: bool,

    /// Additional options for the nix builder (see nix-build(1))
    #[arg(long("nix-option"), num_args = 2)]
    nix_options: Vec<Vec<String>>,
}

impl BuildArgs {
    fn nix_options(&self) -> Vec<(&str, &str)> {
        self.nix_options.iter()
            .flat_map(|v| v.as_chunks::<2>().0.first())
            .map(|s| (s[0].as_str(), s[1].as_str()))
            .collect()
    }
}

fn main() {
    error::resolve(aliases::load_aliases());
    let args = Args::parse();

    use Subcommand::*;
    let result = match args.subcommand {
        Aliases(cmd) => cmd.exec(),
        Build(cmd) => cmd.exec(),
        Completions(cmd) => cmd.exec(),
        Develop(cmd) => cmd.exec(),
        Man(cmd) => cmd.exec(),
        Run(cmd) => cmd.exec(),
        Shell(cmd) => cmd.exec(),
        Show(cmd) => cmd.exec(),
    };

    error::resolve(result)
}
