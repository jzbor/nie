//! Data structures for describing objects in the Nix store which we can work with
//!
//! A [`NixOutput`] comes from a [`NixFile`] which comes from a [`Checkout`].
mod checkout;
mod file;
mod output;

pub use checkout::Checkout;
pub use file::NixFile;
pub use output::NixOutput;
