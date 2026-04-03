use crate::EvalArgs;
use crate::error::NieResult;
use crate::location::{AttributePath, NixReference};
use crate::store::NixOutput;


#[derive(clap::Args)]
pub struct BuildCommand {
    /// Nix references to fetch and build
    #[arg(default_value = ".")]
    refs: Vec<NixReference>,

    /// Build derivations on a remote machine
    ///
    /// The procedure is as follows:
    /// 1. Fetch the sources to the local machine
    /// 2. Copy the sources to the remote machine via [`nix-copy-closure`](https://nix.dev/manual/nix/stable/command-ref/nix-copy-closure.html).
    /// 3. Build the output on the remote machine using ssh and [`nix-build`](https://nix.dev/manual/nix/stable/command-ref/nix-build.html).
    /// 4. Fetch the output from the remote machine via [`nix-copy-closure`](https://nix.dev/manual/nix/stable/command-ref/nix-copy-closure.html).
    ///
    /// *Note that this may require your user to be [trusted](https://nix.dev/manual/nix/stable/command-ref/conf-file.html?highlight=trusted%20user#conf-trusted-users) by the Nix daemon.*
    #[arg(long("on"))]
    remote: Option<String>,

    #[clap(flatten)]
    eval_args: EvalArgs,

    /// Additional arguments for the nix builder (see nix-build(1))
    #[arg(last = true, allow_hyphen_values = true)]
    extra_args: Vec<String>,
}

impl super::Command for BuildCommand {
    fn exec(self) -> NieResult<()> {
        let default = AttributePath::common_package_locations();
        let paths: Vec<_> = NixOutput::fetch_and_build_all(&self.refs, &default, true, &self.eval_args,
                                                            &self.extra_args, self.remote.as_deref())?
            .into_iter()
            .flatten()
            .collect();

        paths.iter().for_each(|p| println!("{}", p.to_string_lossy()));
        Ok(())
    }
}
