use std::process;
use std::time::Instant;

use crate::error::{NieError, NieResult};
use crate::interact::{announce, inform_check_done, inform_check_summary, inform_running_checks};
use crate::location::{AttributePath, NixReference};
use crate::store::Checkout;
use crate::{EvalArgs, nix};


#[derive(clap::Args)]
pub struct CheckCommand {
    /// Nix reference to fetch and check
    #[arg(default_value = ".")]
    reference: NixReference,

    /// Run all checks regardless of failing ones
    #[arg(short, long)]
    keep_going: bool,

    /// Also build packages as part of checks
    #[arg(short, long)]
    packages: bool,

    #[clap(flatten)]
    eval_args: EvalArgs,

    /// Additional arguments for the nix builder (see nix-build(1))
    #[arg(last = true, allow_hyphen_values = true)]
    extra_args: Vec<String>,
}

impl super::Command for CheckCommand {
    fn exec(self) -> NieResult<()> {
        let common = AttributePath::common_check_locations();
        let checkout = Checkout::fetch(self.reference.repository().clone())?;
        let file = checkout.file(self.reference.filename().cloned(), self.eval_args)?;

        let checks = if *self.reference.attribute() != AttributePath::default() {
            vec!(file.output(self.reference.attribute().to_owned(), &common)?)
        } else {
            let check_parent = if file.has_attribute(&AttributePath::from("checks").child(nix::current_system()?))? {
                AttributePath::from("checks").child(nix::current_system()?)
            } else {
                AttributePath::from("checks")
            };
            let package_parent = if file.has_attribute(&AttributePath::from("packages").child(nix::current_system()?))? {
                AttributePath::from("packages").child(nix::current_system()?)
            } else {
                AttributePath::from("packages")
            };

            let potential_checks: Vec<_> = file.attributes(5, false)?
                .filter(|a| a.is_indirect_child(&check_parent) || (self.packages && a.is_indirect_child(&package_parent)))
                .collect();

            // Only consider leaf elements
            potential_checks.iter()
                .filter(|c| !potential_checks.iter().any(|pc| pc.is_indirect_child(c)))
                .map(|a| file.output(a.to_owned(), &[]))
                .collect::<NieResult<_>>()?
        };

        if checks.is_empty() {
            return Err(NieError::NoChecksFound(self.reference.into()))
        }

        inform_running_checks(&checks);

        let start = Instant::now();
        let mut results = Vec::new();
        for check in &checks {
            announce(&format!("Running check \"{}\"", check.reference().attribute()));
            let start_time = Instant::now();
            let result = check.build(None, false, &self.extra_args, None);
            let end_time = Instant::now();
            let success = result.is_ok();
            results.push(result);

            inform_check_done(check.reference().attribute(), success, end_time - start_time);

            if !self.keep_going && !success {
                break;
            }
        }
        let end = Instant::now();

        inform_check_summary(&checks, &results, end - start);

        if results.iter().any(|c| c.is_err()) {
            process::exit(1);
        }

        Ok(())
    }
}
