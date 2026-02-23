use crate::EvalArgs;
use crate::error::NieResult;
use crate::location::NixReference;
use crate::store::checkout::Checkout;


#[derive(clap::Args)]
pub struct EvaluateCommand {
    /// Nix references to fetch and build
    #[arg(default_value = ".")]
    reference: NixReference,

    #[clap(flatten)]
    eval_args: EvalArgs,

    /// Additional arguments for the nix builder (see nix-build(1))
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    extra_args: Vec<String>,
}


impl super::Command for EvaluateCommand {
    fn exec(self) -> NieResult<()> {
        let checkout = Checkout::create(self.reference.repository().clone())?;
        let file = checkout.file(self.reference.filename().cloned(), self.eval_args)?;
        let output = file.output(self.reference.attribute().to_owned(), &[])?;
        let stdout = output.eval(&self.extra_args)?;

        print!("{}", stdout);

        Ok(())
    }
}
