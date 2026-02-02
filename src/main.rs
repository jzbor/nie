use std::collections::HashMap;

use clap::Parser;

use crate::commands::Command;
use crate::error::NieResult;


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

    /// Run checks for a Nix repo
    Check(commands::check::CheckCommand),

    /// Generate shell completions
    #[command(hide = true)]
    Completions(commands::completions::CompletionsCommand),

    /// Build a package from a Nix repo
    Build(commands::build::BuildCommand),

    /// Enter a dev shell from a Nix repo
    #[command(visible_alias = "dev", alias = "dev-shell")]
    Develop(commands::develop::DevelopCommand),

    /// Evaluate an expression from a Nix repo
    #[command(visible_alias = "eval")]
    Evaluate(commands::evaluate::EvaluateCommand),

    /// Initialize a new nix project from a template
    #[command(visible_alias = "init", visible_alias = "initialise")]
    Initialize(commands::initialize::InitializeCommand),

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

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, clap::Args)]
struct EvalArgs {
    /// Enable Flake compatibility
    #[arg(short, long)]
    flake_compat: bool,

    /// Additional options for the nix builder (see nix-build(1))
    #[arg(long("nix-option"), num_args = 2)]
    nix_options: Vec<Vec<String>>,

    /// `true` if the targeted expression is not a lambda at the toplevel
    ///
    /// Calculated on the fly.
    // #[arg(skip)]
    #[arg(long)]
    is_lambda: bool,

    /// Additional arguments to pass to the expression at the toplevel
    #[arg(long("arg"), num_args = 2)]
    add_eval_args: Vec<Vec<String>>,

    /// Pass 'system' argument to the expression
    ///
    /// As a flag this provides builtins.currentSystem, otherwise it is possible to provide a
    /// custom value.
    #[arg(long)]
    system: Option<Option<String>>,
}


impl EvalArgs {
    fn nix_options(&self) -> Vec<(&str, &str)> {
        self.nix_options.iter()
            .flat_map(|v| v.as_chunks::<2>().0.first())
            .map(|s| (s[0].as_str(), s[1].as_str()))
            .collect()
    }

    fn expression_args(&self) -> Vec<(&str, &str)> {
        self.add_eval_args.iter()
            .flat_map(|v| v.as_chunks::<2>().0.first())
            .map(|s| (s[0].as_str(), s[1].as_str()))
            .collect()
    }

    fn expression_args_str(&self) -> NieResult<String> {
        if !self.is_lambda {
            return Ok(String::new());
        }

        let mut s = String::new();
        let mut map: HashMap<_, _> = self.expression_args().into_iter()
            .collect();

        #[allow(unused_assignments)]
        let mut current_system = String::new();
        match &self.system {
            Some(None) => {
                current_system = format!("\"{}\"", nix::current_system()?);
                map.insert("system", &current_system);
            },
            Some(Some(s)) => {
                current_system = format!("\"{}\"", s.as_str());
                map.insert("system", current_system.as_str());
            },
            None => (),
        }


        s.push_str("{ ");

        for (key, value) in map {
            s.push_str(key);
            s.push_str(" = ");
            s.push_str(value);
            s.push_str("; ");
        }

        s.push_str("} ");

        Ok(s)
    }
}


fn main() {
    error::resolve(aliases::load_aliases());
    let args = Args::parse();

    use Subcommand::*;
    let result = match args.subcommand {
        Aliases(cmd) => cmd.exec(),
        Build(cmd) => cmd.exec(),
        Check(cmd) => cmd.exec(),
        Completions(cmd) => cmd.exec(),
        Develop(cmd) => cmd.exec(),
        Evaluate(cmd) => cmd.exec(),
        Initialize(cmd) => cmd.exec(),
        Man(cmd) => cmd.exec(),
        Run(cmd) => cmd.exec(),
        Shell(cmd) => cmd.exec(),
        Show(cmd) => cmd.exec(),
    };

    error::resolve(result)
}
