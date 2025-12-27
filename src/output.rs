use std::path::PathBuf;
use std::sync::{Arc, RwLock};

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

    pub fn file(&self) -> NixFile {
        self.0.read().unwrap().file.clone()
    }

    pub fn attr(&self) -> AttributePath {
        self.0.read().unwrap().attr.clone()
    }

    pub fn build(&self, out_link: bool, extra_args: &[String]) -> NieResult<Vec<PathBuf>> {
        let attr = self.attr().clone();
        let path = self.file().path();

        if let Some(paths) = &self.0.read().unwrap().built_paths {
            Ok(paths.clone())
        } else {
            announce(&format!("Building {} from {}", attr.to_string_user(), path.to_string_lossy()));
            let paths = nix::build(&path, &attr, out_link, extra_args)?;
            self.0.write().unwrap().built_paths = Some(paths.clone());
            Ok(paths)
        }
    }
}
