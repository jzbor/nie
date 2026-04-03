//! Representation of Nix attribute paths

use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use crate::error::NieError;
use crate::nix;

/// Representation of Nix attribute paths as a chain of attribute keys
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct AttributePath(Vec<String>);


impl AttributePath {
    /// Create a new [`AttributePath`] from a child entry of the path `self`.
    pub fn child(&self, name: String) -> Self {
        let mut new_path = self.0.clone();
        new_path.push(name);
        AttributePath(new_path)
    }

    /// Create a new [`AttributePath`] from the parent of `self`.
    ///
    /// May return [`None`] if `self` is already a top-level path.
    pub fn parent(&self) -> Option<Self> {
        if *self == Self::default() {
            return None
        }

        let mut new_path = self.0.clone();
        new_path.pop();
        Some(AttributePath(new_path))
    }

    /// Concatenate two [`AttributePath`]s
    pub fn join(&self, other: &Self) -> Self {
        let mut new_path = self.0.clone();
        new_path.extend(other.0.clone());
        AttributePath(new_path)
    }

    /// Check if `self` is a direct child of `potential_parent`.
    ///
    /// [`true`] if `potential_parent` consists exactly of the non-terminal components of `self`.
    /// [`false`] otherwise.
    /// Toplevel paths have no parent, therefore are child to no other path.
    ///
    /// See also [`AttributePath::is_indirect_child()`].
    pub fn is_child(&self, potential_parent: &Self) -> bool {
        self.parent()
            .map(|p| p == *potential_parent)
            .unwrap_or_default()
    }

    /// Check if `self` is a indirect child of `potential_parent`.
    ///
    /// [`true`] if `potential_parent` consists of some number of preceding components of `self`
    /// [`false`] otherwise.
    /// Toplevel paths have no parent, therefore are indirect child to no other path.
    ///
    /// See also [`AttributePath::is_child()`].
    pub fn is_indirect_child(&self, potential_parent: &Self) -> bool {
        self.is_child(potential_parent)
            || self.parent()
                .map(|p| p.is_indirect_child(potential_parent))
                .unwrap_or_default()
    }

    /// The number of components in `self`.
    pub fn depth(&self) -> usize {
        self.len()
    }

    /// [`true`] if `self` is a toplevel path, i.e. has no components.
    pub fn is_toplevel(&self) -> bool {
        self.is_empty()
    }

    /// The last component of `self`.
    pub fn name(&self) -> Option<&str> {
        self.last().map(|l| l.as_str())
    }

    /// Convert to a user-readable, not necessarily machine-readable, [`String`].
    pub fn to_string_user(&self) -> String {
        if self.is_toplevel() {
            String::from("<toplevel>")
        } else {
            self.to_string()
        }
    }

    /// [`AttributePath`]s whose children commonly contain checks.
    pub fn common_check_locations() -> Vec<AttributePath> {
        let mut pkgs = vec![
            AttributePath::from("checks"),
        ];

        if let Ok(system) = nix::current_system() {
            pkgs.push(AttributePath::from(format!("checks.{}", system).as_str()));
        }

        pkgs.push(AttributePath::default());

        pkgs
    }

    /// [`AttributePath`]s whose children commonly contain templates.
    pub fn common_template_locations() -> Vec<AttributePath> {
        let mut pkgs = vec![
            AttributePath::from("templates"),
        ];

        if let Ok(system) = nix::current_system() {
            pkgs.push(AttributePath::from(format!("templates.{}", system).as_str()));
        }

        pkgs.push(AttributePath::default());

        pkgs
    }

    /// [`AttributePath`]s whose children commonly contain consumable packages.
    pub fn common_package_locations() -> Vec<AttributePath> {
        let mut pkgs = vec![
            AttributePath::from("packages"),
            AttributePath::from("legacyPackages"),
        ];

        if let Ok(system) = nix::current_system() {
            pkgs.push(AttributePath::from(format!("packages.{}", system).as_str()));
            pkgs.push(AttributePath::from(format!("legacyPackages.{}", system).as_str()));
        }

        pkgs.push(AttributePath::default());

        pkgs
    }

    /// [`AttributePath`]s whose children commonly contain development shell definitions.
    pub fn common_dev_shell_locations() -> Vec<AttributePath> {
        let mut pkgs = vec![
            AttributePath::from("devShells"),
        ];

        if let Ok(system) = nix::current_system() {
            pkgs.push(AttributePath::from(format!("devShells.{}", system).as_str()));
        }

        pkgs.push(AttributePath::default());


        pkgs
    }
}


impl Deref for AttributePath {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AttributePath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for AttributePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("."))
    }
}

impl FromStr for AttributePath {
    type Err = NieError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}

impl From<&str> for AttributePath {
    fn from(value: &str) -> Self {
        if value.is_empty() {
            Self::default()
        } else {
            AttributePath(value.split(".").map(|s| s.to_owned()).collect())
        }
    }
}
