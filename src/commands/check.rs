use std::time::Instant;

use colored::Colorize;

use crate::{EvalArgs, nix};
use crate::attribute_path::AttributePath;
use crate::error::{NieError, NieResult};
use crate::interaction::announce;
use crate::location::NixReference;
use crate::store::checkout::Checkout;


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
        let checkout = Checkout::create(self.reference.repository().clone())?;
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

        println!();
        announce(&format!("Running {} checks:", checks.len()));
        for check in &checks {
            println!("  🔳 {}", check.reference().attribute());
        }
        println!();

        let start = Instant::now();
        let mut results = Vec::new();
        for check in &checks {
            announce(&format!("Running check \"{}\"", check.reference().attribute()));
            let start_time = Instant::now();
            let result = check.build(false, &self.extra_args, None);
            let end_time = Instant::now();
            let is_err = result.is_err();
            results.push(result);

            if is_err {
                println!("{} to build \"{}\" (took {:?})", "FAILED".red().bold(),
                    check.reference().attribute(), end_time - start_time)
            } else {
                println!("{} built \"{}\" (took {:?})", "SUCCESSFULLY".green().bold(),
                    check.reference().attribute(), end_time - start_time)
            }

            if !self.keep_going && is_err {
                break;
            }
        }
        let end = Instant::now();

        println!();
        announce(&format!("Results (took {:?}):", end - start));
        for (i, check) in checks.into_iter().enumerate() {
            let result = results.get(i);
            match result {
                Some(Ok(_)) => println!("  ✅ {}", check.reference().attribute()),
                Some(Err(_)) => println!("  ❌ {}", check.reference().attribute()),
                None => println!("  ❔ {}", check.reference().attribute()),
            }
        }
        println!();
        if results.iter().any(Result::is_err) {
            println!("    {}", "FAILURE".red().bold())
        } else {
            println!("    {}", "SUCCESS".green().bold())
        }
        println!();

        Ok(())
    }
}
