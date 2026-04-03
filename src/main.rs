#![doc = include_str!("../README.md")]

use std::collections::HashMap;

use clap::Parser;

use crate::commands::Command;
use crate::error::NieResult;


mod commands;
mod error;
mod interact;
mod location;
mod store;
mod nix;
mod registry;
mod aliases;


const ENV_TRACE_EXEC: &str = "NIE_TRACE_EXEC";


#[derive(Parser)]
#[command(version, author, name = "nie")]
/// Alternative Nix CLI to run and build remote derivations.
///
/// ## Resource Identifiers
/// `nie` addresses repositories with resource identifiers somewhat similar to Flake references and
/// URIs.
/// Those resource identifiers consist of multiple tokens separated by a number sign (`#`).
/// The first of those tokens is expected to be a [location reference](#resource-identifiers-location-references).
/// Any following token may be an [output selector](#resource-identifiers-output-selector)
/// or a [key-value argument](#resource-identifiers-key-value-arguments).
///
/// ### Resource Identifiers - Examples
/// ```text
/// github://nixos/nixpkgs/nixos-unstable#hello
/// git@github.com:nixos/nixpkgs#coreutils
/// codeberg://mergiraf/mergiraf
/// ```
///
/// ### Resource Identifiers - Location References
/// * **Git** (`git://<git_url>`):
///   Uses [`fetchGit`](https://nix.dev/manual/nix/stable/language/builtins.html#builtins-fetchGit)
///   to fetch a repository from an arbitrary git source.
///   Sources are cloned shallowly by default and includes submodules.
///
/// * **Tarballs** (`<url>`):
///   Uses the [`fetchTarball`](https://nix.dev/manual/nix/stable/language/builtins.html#builtins-fetchTarball)
///   to fetch a remote tarball.
///
/// * **Local** (`file://<path>`):
///   Uses [`nix-store --add`](https://nix.dev/manual/nix/stable/command-ref/nix-store/add.html)
///   copy a local directory into the store.
///
/// * **Codeberg** (`codeberg://<owner>/<repo>/<ref>`):
///   Uses the [`fetchTarball`](https://nix.dev/manual/nix/stable/language/builtins.html#builtins-fetchTarball)
///   to fetch a repository from a codeberg.org endpoint.
///   Falls back to [`fetchGit`](https://nix.dev/manual/nix/stable/language/builtins.html#builtins-fetchGit).
///
/// * **GitHub** (`github://<owner>/<repo>/<branch>`):
///   Uses the [`fetchTarball`](https://nix.dev/manual/nix/stable/language/builtins.html#builtins-fetchTarball)
///   to fetch a repository from a github.com endpoint.
///   Falls back to [`fetchGit`](https://nix.dev/manual/nix/stable/language/builtins.html#builtins-fetchGit).
///
/// If no specific prefix is given, `nie` tries to guess the desired location.
///
/// ### Resource Identifiers - Output Selector
/// The non-initial token that does not contain an equals sign (`=`) is regarded as the output
/// selector.
/// It specifies which output/derivation of the repository should be used/built.
/// If there is no exact match `nie` tries to guess the appropriate path (e.g. by trying to prefix
/// `packages.<system>` or similar common paths.
///
/// ### Resource Identifiers - Key-Value Arguments
/// Other tokens can specify key-value-arguments in the form `<key>=<value>`:
///
/// * `file` (or short `f`): Specify which file from the repo shall be used.
///
/// * Git fetcher: all arguments for
///   [`fetchGit`](https://nix.dev/manual/nix/stable/language/builtins.html#builtins-fetchGit)
///   can be overwritten.
///
/// * Tarball fetcher: all arguments for
///   [`fetchTarball`](https://nix.dev/manual/nix/stable/language/builtins.html#builtins-fetchTarball)
///   can be overwritten.
///
/// * Local fetcher: pass `nocopy=true` to avoid copying the path to the store.
///
/// * Codeberg fetcher: `ref` can be overwritten with as k/v argument. If the fetcher falls back to
///   [`fetchGit`](https://nix.dev/manual/nix/stable/language/builtins.html#builtins-fetchGit)
///   all of its arguments can also be overwritten.
///
/// * Github fetcher: `branch` can be overwritten with as k/v argument. `tag` can be given specified
///   as k/v argument. If the fetcher falls back to
///   [`fetchGit`](https://nix.dev/manual/nix/stable/language/builtins.html#builtins-fetchGit)
///   all of its arguments can also be overwritten.
///
/// ## Aliases
/// `nie` allows users to specify aliases for [locations](#resource-identifiers-location-references).
/// They are stored as simple space-separated key-value pairs in `$XDG_CONFIG_DIRS/nie/aliases.txt` and
/// `$XDG_CONFIG_HOME/nie/aliases.txt`.
///
/// You can interactively manage those aliases via the `aliases` subcommand.
///
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

    /// `true` if the targeted expression is a lambda at the toplevel
    ///
    /// Calculated on the fly.
    #[arg(skip)]
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
