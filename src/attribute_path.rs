use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use crate::error::NieError;
use crate::nix;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct AttributePath(Vec<String>);


impl AttributePath {
    pub fn child(&self, name: String) -> Self {
        let mut new_path = self.0.clone();
        new_path.push(name);
        AttributePath(new_path)
    }

    pub fn join(&self, other: &Self) -> Self {
        let mut new_path = self.0.clone();
        new_path.extend(other.0.clone());
        AttributePath(new_path)
    }

    pub fn depth(&self) -> usize {
        self.len() - 1
    }

    pub fn is_toplevel(&self) -> bool {
        self.is_empty()
    }

    pub fn name(&self) -> Option<&str> {
        self.last().map(|l| l.as_str())
    }

    pub fn to_string_user(&self) -> String {
        if self.is_toplevel() {
            String::from("<toplevel>")
        } else {
            self.to_string()
        }
    }

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

    pub fn common_package_locations() -> Vec<AttributePath> {
        let mut pkgs = vec![
            AttributePath::from("packages"),
        ];

        if let Ok(system) = nix::current_system() {
            pkgs.push(AttributePath::from(format!("packages.{}", system).as_str()));
        }

        pkgs.push(AttributePath::default());

        pkgs
    }

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
