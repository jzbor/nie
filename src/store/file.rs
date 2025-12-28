use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

use crate::error::{NieError, NieResult};
use crate::location::{AttributePath, NixFileReference};
use crate::store::checkout::Checkout;
use crate::store::output::NixOutput;
use crate::nix;
use crate::registry::Registry;

static FILE_REGISTRY: Registry<NixFileReference, NixFile> = Registry::new();

#[derive(Clone)]
pub struct NixFile(Arc<InnerNixFile>);

struct InnerNixFile {
    checkout: Checkout,
    filename: Option<PathBuf>,
}

pub struct AttributeIterator<'a> {
    _file: &'a NixFile,
    attributes: VecDeque<AttributePath>,
}


impl NixFile {
    pub fn new(checkout: Checkout, filename: Option<PathBuf>) -> NieResult<Self> {
        let key = checkout.repository().with_file(filename.clone());
        if let Some(file) = FILE_REGISTRY.lookup(&key) {
            return Ok(file);
        }

        let file = NixFile(Arc::new(InnerNixFile {
            checkout,
            filename,
        }));

        if !file.path().exists() {
            return Err(NieError::NixFileNotFound(
                    file.0.filename.as_ref().map(|f| f.to_string_lossy().to_string()).unwrap_or_default(),
                    file.0.checkout.path().to_string_lossy().to_string()
            ))
        }

        FILE_REGISTRY.store(key, file.clone());

        Ok(file)
    }

    pub fn fetch(reference: &NixFileReference) -> NieResult<Self> {
        let checkout = Checkout::create(reference.repository().clone())?;
        checkout.file(reference.filename().cloned())
    }

    pub fn output(&self, attr: AttributePath) -> NieResult<NixOutput> {
        NixOutput::new(self.clone(), attr)
    }

    pub fn outputs(files: impl IntoIterator<Item = (Self, AttributePath)>) -> NieResult<Vec<NixOutput>> {
        files.into_iter()
            .map(|(f, a)| f.output(a.clone()))
            .collect()
    }

    pub fn reference(&self) -> NixFileReference {
        self.0.checkout.repository().with_file(self.0.filename.clone())
    }

    pub fn attributes(&self, depth: u32, reject_broken: bool) -> NieResult<AttributeIterator<'_>> {
        let full_expr = include_str!("../nix/discover.nix");
        let value = nix::exec_output_json("nix-instantiate", [
            "--eval",
            "--raw",
            "-E", full_expr,
            "--arg", "path", self.path().to_string_lossy().to_string().as_str(),
            "--arg", "maxdepth", depth.to_string().as_str(),
        ])?;

        let attributes = Self::unfold_attributes(vec!(), AttributePath::default(), value, reject_broken)?.into();

        Ok(AttributeIterator {
            _file: self,
            attributes,
        })
    }

    fn unfold_attributes(mut acc: Vec<AttributePath>, parent: AttributePath,
            value: serde_json::Value, reject_broken: bool) -> NieResult<Vec<AttributePath>> {
        use serde_json::Value::*;
        let map = match value {
            Object(map) => map,
            String(s) => if s.as_str() == "<broken>" && !reject_broken {
                return Ok(acc)
            } else{
                return Err(NieError::BrokenAttribute(parent))
            },
            _ => return Err(NieError::JsonUnfolding(value)),
        };

        for (k, v) in map {
            let new = parent.child(k);
            acc.push(new.clone());
            acc = Self::unfold_attributes(acc, new, v, reject_broken)?;
        }

        Ok(acc)
    }

    pub fn path(&self) -> PathBuf {
        match &self.0.filename {
            Some(filename) => self.0.checkout.path().join(filename),
            None => self.0.checkout.path().to_owned(),
        }
    }
}


impl Iterator for AttributeIterator<'_> {
    type Item = AttributePath;

    fn next(&mut self) -> Option<Self::Item> {
        self.attributes.pop_front()
    }
}
