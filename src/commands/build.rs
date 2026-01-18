use crate::EvalArgs;
use crate::attribute_path::AttributePath;
use crate::error::NieResult;
use crate::location::NixReference;
use crate::store::output::NixOutput;


#[derive(clap::Args)]
pub struct BuildCommand {
    /// Nix references to fetch and build
    #[arg(default_value = "./.")]
    refs: Vec<NixReference>,

    #[clap(flatten)]
    eval_args: EvalArgs,

    /// Additional arguments for the nix builder (see nix-build(1))
    #[arg(last = true, allow_hyphen_values = true)]
    extra_args: Vec<String>,
}

impl super::Command for BuildCommand {
    fn exec(self) -> NieResult<()> {
        let default = AttributePath::common_package_locations();
        let paths: Vec<_> = NixOutput::fetch_and_build_all(&self.refs, &default, true, &self.eval_args, &self.extra_args)?
            .into_iter()
            .flatten()
            .collect();

        paths.iter().for_each(|p| println!("{}", p.to_string_lossy()));
        Ok(())
    }
}
