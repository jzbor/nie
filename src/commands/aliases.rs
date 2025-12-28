use std::fs::{File, OpenOptions};
use std::io::Write;
use std::str::FromStr;

use crate::aliases;
use crate::error::{NieError, NieResult};
use crate::location::NixReference;


#[derive(clap::Args)]
pub struct AliasesCommand {
    /// Add a new alias for the user
    #[clap(long, exclusive = true, num_args = 2)]
    add: Option<Vec<String>>,

    /// Query for an existing alias
    #[clap(short, long, exclusive = true)]
    query: Option<String>,

    /// List all considered files containing aliases
    #[clap(long, exclusive = true)]
    files: bool,

    /// Arguments passed to command
    #[clap(last = true)]
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
            write!(file, "{} {}\n", src, dest)?;
        } else if let Some(query) = self.query {
            let aliases = aliases::load_aliases()?;
            let dest = aliases.get(&query)
                .ok_or(NieError::AliasNotFound(query))?;
            println!("{}", dest);
        } else if self.files {
            for file in aliases::alias_files() {
                println!("{}", file.to_string_lossy());
            }
        } else {
            for (k, v) in aliases::load_aliases()? {
                println!("{} {}", k, v);
            }
        }

        Ok(())
    }
}
