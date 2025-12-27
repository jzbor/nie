use std::iter;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::checkout::Checkout;
use crate::error::{NieError, NieResult};
use crate::file::NixFile;
use crate::interaction::announce;
use crate::location::{AttributePath, NixReference};
use crate::nix;
use crate::registry::Registry;


static OUTPUT_REGISTRY: Registry<NixReference, NixOutput> = Registry::new();

#[derive(Clone)]
pub struct NixOutput(Arc<RwLock<InnerNixOutput>>);

struct InnerNixOutput {
    file: NixFile,
    attr: AttributePath,
    built_paths: Option<Vec<PathBuf>>,
}


impl NixOutput {
    pub fn new(file: NixFile, attr: AttributePath) -> NieResult<Self> {
        let key = file.reference().with_attribute(attr.clone());
        if let Some(file) = OUTPUT_REGISTRY.lookup(&key) {
            return Ok(file);
        }

        if !attr.is_toplevel() && !nix::has_attribute(&file.path(), &attr)? {
            return Err(NieError::AttributeNotFound(file.path().to_string_lossy().to_string(), attr))
        }


        let output = NixOutput(Arc::new(RwLock::new(InnerNixOutput {
            file, attr,
            built_paths: None,
        })));

        OUTPUT_REGISTRY.store(key, output.clone());

        Ok(output)
    }

    pub fn fetch_and_build(reference: &NixReference, out_links: bool, nix_args: &[String]) -> NieResult<Vec<PathBuf>> {
        let checkout = Checkout::create(reference.repository().clone())?;
        let file = checkout.file(reference.filename().cloned())?;
        let output = file.output(reference.attribute().clone())?;
        output.build(out_links, nix_args)
    }

    pub fn fetch_and_build_all(refs: &[NixReference], out_links: bool, nix_args: &[String]) -> NieResult<Vec<Vec<PathBuf>>> {
        let repo_refs = refs.iter().map(|s| s.repository()).cloned();
        let filenames = refs.iter().map(|s| s.filename().cloned());
        let attributes = refs.iter().map(|s| s.attribute()).cloned();
        let checkouts = Checkout::create_all(repo_refs)?;
        let files = Checkout::files(iter::zip(checkouts.iter().cloned(), filenames))?;
        let outputs = NixFile::outputs(iter::zip(files.iter().cloned(), attributes))?;
        outputs.into_iter()
            .map(|o| o.build(out_links, nix_args))
            .collect::<NieResult<Vec<_>>>()
    }

    pub fn file(&self) -> NixFile {
        self.0.read().unwrap().file.clone()
    }

    pub fn attr(&self) -> AttributePath {
        self.0.read().unwrap().attr.clone()
    }

    pub fn drv_name(&self) -> NieResult<String> {
        let paths = self.build(false, &[])?;
        let path = paths.first()
            .ok_or(NieError::NoOutputPath(Box::new(self.reference())))?;

        let name = path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .split("-")
            .skip(1)
            .take_while(|s| s.starts_with(|c: char| c.is_alphabetic()))
            .collect::<Vec<_>>()
            .join("-");

        Ok(name)
    }

    pub fn reference(&self) -> NixReference {
        self.file().reference().with_attribute(self.attr())
    }

    pub fn build(&self, out_links: bool, extra_args: &[String]) -> NieResult<Vec<PathBuf>> {
        let attr = self.attr().clone();
        let path = self.file().path();

        if let Some(paths) = &self.0.read().unwrap().built_paths {
            Ok(paths.clone())
        } else {
            announce(&format!("Building {} from {}", attr.to_string_user(), path.to_string_lossy()));
            let paths = nix::build(&path, &attr, out_links, extra_args)?;
            self.0.write().unwrap().built_paths = Some(paths.clone());
            Ok(paths)
        }
    }
}
