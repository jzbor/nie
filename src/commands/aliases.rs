use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::str::FromStr;

use crate::{aliases, nix};
use crate::error::{NieError, NieResult};
use crate::location::NixReference;


#[derive(clap::Args)]
pub struct AliasesCommand {
    /// Add a new alias for the user
    #[arg(long, exclusive = true, num_args = 2)]
    add: Option<Vec<String>>,

    /// Query for an existing alias
    #[arg(short, long, exclusive = true)]
    query: Option<String>,

    /// List all considered files containing aliases
    #[arg(long, exclusive = true)]
    files: bool,

    /// Open aliases file in $EDITOR
    #[arg(long, exclusive = true)]
    edit: bool,

    /// Arguments passed to command
    #[arg(last = true)]
    args: Vec<String>,
}

impl super::Command for AliasesCommand {
    fn exec(self) -> NieResult<()> {

        if let Some(add_args) = self.add {
            let src = &add_args[0];
            let dest = NixReference::from_str(&add_args[1])?;
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(aliases::user_alias_file()?)?;
            writeln!(file, "{} {}", src, dest)?;
        } else if let Some(query) = self.query {
            let aliases = aliases::load_aliases()?;
            let dest = aliases.get(&query)
                .ok_or(NieError::AliasNotFound(query))?;
            println!("{}", dest);
        } else if self.files {
            for file in aliases::alias_files() {
                println!("{}", file.to_string_lossy());
            }
        } else if self.edit {
            let path = aliases::user_alias_file()?;
            let editor = env::var("EDITOR")
                .map_err(|_| NieError::EnvVarMissing("EDITOR"))?;
            nix::exec(&editor, &[&path.to_string_lossy().to_string()])?;
        } else {
            for (k, v) in aliases::load_aliases()? {
                println!("{} {}", k, v);
            }
        }

        Ok(())
    }
}
