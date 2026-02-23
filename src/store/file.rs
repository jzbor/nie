use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::attribute_path::AttributePath;
use crate::error::{NieError, NieResult};
use crate::location::NixFileReference;
use crate::store::checkout::Checkout;
use crate::store::output::NixOutput;
use crate::{EvalArgs, nix};
use crate::registry::Registry;

static FILE_REGISTRY: Registry<(NixFileReference, EvalArgs), NixFile> = Registry::new();

#[derive(Clone)]
pub struct NixFile(Arc<RwLock<InnerNixFile>>);

struct InnerNixFile {
    checkout: Checkout,
    filename: Option<PathBuf>,
    cached_attributes: HashMap<AttributePath, bool>,
    eval_args: EvalArgs,
}

pub struct AttributeIterator<'a> {
    _file: &'a NixFile,
    attributes: VecDeque<AttributePath>,
}


impl NixFile {
    pub fn new(checkout: Checkout, filename: Option<PathBuf>, mut eval_args: EvalArgs) -> NieResult<Self> {
        let key = (checkout.repository().with_filename(filename.clone()), eval_args.clone());
        if let Some(file) = FILE_REGISTRY.lookup(&key) {
            return Ok(file);
        }

        let checkout_dir = checkout.path();
        let expected_file = match &filename {
            Some(filename) => checkout.path().join(filename),
            None => checkout_dir.join("default.nix"),
        };
        let requires_flake_compat = fs::exists(checkout_dir.join("flake.nix"))?
            && !fs::exists(&expected_file)?;
        let cached_attributes = HashMap::default();

        eval_args.flake_compat |= requires_flake_compat;
        if !eval_args.flake_compat {
            eval_args.is_lambda = nix::is_lambda(&expected_file)?;
        }


        let file = NixFile(Arc::new(RwLock::new(InnerNixFile {
            checkout,
            filename,
            cached_attributes,
            eval_args,
        })));

        if !file.path().exists() {
            let read = file.0.read().unwrap();
            return Err(NieError::NixFileNotFound(
                    read.filename.as_ref().map(|f| f.to_string_lossy().to_string()).unwrap_or_default(),
                    read.checkout.path().to_string_lossy().to_string()
            ))
        }

        FILE_REGISTRY.store(key, file.clone());

        Ok(file)
    }

    pub fn flake_compat(&self) -> bool {
        self.0.read().unwrap().eval_args.flake_compat
    }

    pub fn eval_args(&self) -> EvalArgs {
        self.0.read().unwrap().eval_args.clone()
    }

    pub fn checkout(&self) -> Checkout {
        self.0.read().unwrap().checkout.clone()
    }

    pub fn fetch(reference: &NixFileReference, eval_args: EvalArgs) -> NieResult<Self> {
        let checkout = Checkout::create(reference.repository().clone())?;
        checkout.file(reference.filename().cloned(), eval_args)
    }

    pub fn output(&self, mut attr: AttributePath, common_locations: &[AttributePath]) -> NieResult<NixOutput> {
        if attr == AttributePath::default() {
            for d in common_locations.iter().map(|l| l.child("default".to_owned())) {
                if self.has_attribute(&d).unwrap_or_default() {
                    attr = d.to_owned();
                    break
                }
            }
        } else if !self.has_attribute(&attr).unwrap_or(true) {
            for d in common_locations.iter().map(|l| l.join(&attr)) {
                if self.has_attribute(&d).unwrap_or_default() {
                    attr = d.to_owned();
                    break
                }
            }
        }

        NixOutput::new(self.clone(), attr)
    }

    pub fn outputs(files: impl IntoIterator<Item = (Self, AttributePath)>, common_locations: &[AttributePath])
            -> NieResult<Vec<NixOutput>> {
        files.into_iter()
            .map(|(f, a)| f.output(a.clone(), common_locations))
            .collect()
    }

    pub fn reference(&self) -> NixFileReference {
        let filename = self.0.read().unwrap().filename.clone();
        self.0.read().unwrap().checkout.repository().with_filename(filename)
    }

    pub fn has_attribute(&self, attr: &AttributePath) -> NieResult<bool> {
        if let Some(cached) = self.0.read().unwrap().cached_attributes.get(attr) {
            return Ok(*cached);
        }

        let res = nix::has_attribute(&self.path(), attr, &self.eval_args());

        if let Ok(b) = res {
            self.0.write().unwrap().cached_attributes.insert(attr.to_owned(), b);
        }

        res
    }

    pub fn attributes(&self, depth: u32, reject_broken: bool) -> NieResult<AttributeIterator<'_>> {
        let full_expr = if self.flake_compat() {
            include_str!("../nix/discover_flake.nix")
        } else {
            include_str!("../nix/discover.nix")
        };

        let value = nix::exec_output_json("nix-instantiate", [
            "--eval",
            "--raw",
            "-E", full_expr,
            "--arg", "path", self.path().to_string_lossy().to_string().as_str(),
            "--arg", "maxdepth", depth.to_string().as_str(),
        ])?;

        let attributes = if self.flake_compat() {
            Self::unfold_attributes_flake(vec!(), value, reject_broken)?.into()
        } else {
            Self::unfold_attributes(vec!(), AttributePath::default(), value, reject_broken)?.into()
        };

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

    fn unfold_attributes_flake(mut acc: Vec<AttributePath>,
            value: serde_json::Value, reject_broken: bool) -> NieResult<Vec<AttributePath>> {
        use serde_json::Value::*;
        if let Array(arr) = &value {
            for elem in arr {
                if let Object(map) = elem {
                    let name = match map.get("name") {
                        Some(String(name)) => name,
                        _ => return Err(NieError::JsonUnfolding(value)),
                    };
                    let value = match map.get("value") {
                        Some(value) => value,
                        None => return Err(NieError::JsonUnfolding(value)),
                    };
                    let new = AttributePath::default().child(name.to_string());
                    acc.push(new.clone());
                    acc = Self::unfold_attributes(acc, new, value.clone(), reject_broken)?;
                } else {
                    return Err(NieError::JsonUnfolding(value))
                }
            }
            Ok(acc)
        } else {
            Err(NieError::JsonUnfolding(value))
        }
    }

    pub fn path(&self) -> PathBuf {
        match &self.0.read().unwrap().filename {
            Some(filename) => self.0.read().unwrap().checkout.path().join(filename),
            None => self.0.read().unwrap().checkout.path().to_owned(),
        }
    }
}


impl Iterator for AttributeIterator<'_> {
    type Item = AttributePath;

    fn next(&mut self) -> Option<Self::Item> {
        self.attributes.pop_front()
    }
}
