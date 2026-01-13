use crate::{BuildArgs, nix};
use crate::attribute_path::AttributePath;
use crate::error::NieResult;
use crate::interaction::announce;
use crate::location::NixReference;
use crate::store::checkout::Checkout;


#[derive(clap::Args)]
pub struct CheckCommand {
    /// Nix reference to fetch and check
    #[arg(default_value = "./.")]
    reference: NixReference,

    #[clap(flatten)]
    build_args: BuildArgs,

    /// Additional arguments for the nix builder (see nix-build(1))
    #[arg(last = true, allow_hyphen_values = true)]
    extra_args: Vec<String>,
}

impl super::Command for CheckCommand {
    fn exec(self) -> NieResult<()> {
        let common = AttributePath::common_check_locations();
        let checkout = Checkout::create(self.reference.repository().clone())?;
        let file = checkout.file(self.reference.filename().cloned(), self.build_args.flake_compat)?;

        let checks = if *self.reference.attribute() != AttributePath::default() {
            vec!(file.output(self.reference.attribute().to_owned(), &common)?)
        } else {
            let prefix = if file.has_attribute(&AttributePath::from("checks").child(nix::current_system()?))? {
                AttributePath::from("checks").child(nix::current_system()?)
            } else {
                AttributePath::from("checks")
            };
            let prefix_str = prefix.to_string();
            let potential_checks: Vec<_> = file.attributes(5, false)?
                .filter(|a| a.to_string().starts_with(prefix_str.as_str()))
                .collect();
            // Only consider leaf elements
            potential_checks.iter()
                .filter(|c| !potential_checks.iter().any(|pc| pc != *c && pc.to_string().starts_with(&c.to_string())))
                .map(|a| file.output(a.to_owned(), &[]))
                .collect::<NieResult<_>>()?
        };

        for check in checks {
            announce(&format!("Running check \"{}\"", check.reference().attribute()));
            check.build(false, &self.build_args, &self.extra_args)?;
        }

        Ok(())
    }
}
