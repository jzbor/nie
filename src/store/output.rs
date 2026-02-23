use std::iter;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::attribute_path::AttributePath;
use crate::error::{NieError, NieResult};
use crate::interaction::inform;
use crate::location::NixReference;
use crate::store::checkout::Checkout;
use crate::store::file::NixFile;
use crate::{EvalArgs, nix};
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
        let key= file.reference().with_attribute(attr.clone());
        if let Some(file) = OUTPUT_REGISTRY.lookup(&key) {
            return Ok(file);
        }

        if !attr.is_toplevel() && !file.has_attribute(&attr)? {
            return Err(NieError::AttributeNotFound(file.path().to_string_lossy().to_string(), attr))
        }

        let output = NixOutput(Arc::new(RwLock::new(InnerNixOutput {
            file, attr,
            built_paths: None,
        })));

        OUTPUT_REGISTRY.store(key, output.clone());

        Ok(output)
    }

    pub fn fetch_and_build_all(refs: &[NixReference], common_locations: &[AttributePath], allow_out_links: bool,
            eval_args: &EvalArgs, extra_args: &[String], remote: Option<&str>) -> NieResult<Vec<Vec<PathBuf>>> {
        let repo_refs = refs.iter().map(|s| s.repository()).cloned();
        let filenames = refs.iter().map(|s| s.filename().cloned());
        let attributes = refs.iter().map(|s| s.attribute()).cloned();

        let checkouts = Checkout::create_all(repo_refs)?;
        let files = Checkout::files(iter::zip(checkouts.iter().cloned(), filenames), eval_args.clone())?;
        let outputs = NixFile::outputs(iter::zip(files.iter().cloned(), attributes), common_locations)?;

        outputs.into_iter()
            .map(|o| o.build(allow_out_links, extra_args, remote)
                .and_then(|p| if p.is_empty() {
                    Err(NieError::NoOutputPath(o.reference().into()))
                } else {
                    Ok(p)
                })
            )
            .collect::<NieResult<Vec<_>>>()
    }

    pub fn file(&self) -> NixFile {
        self.0.read().unwrap().file.clone()
    }

    pub fn attr(&self) -> AttributePath {
        self.0.read().unwrap().attr.clone()
    }

    pub fn drv_name(&self) -> NieResult<String> {
        let paths = self.build(false, &[], None)?;
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

    pub fn main_program(&self) -> Option<PathBuf> {
        let self_read = self.0.read().unwrap();
        let built_paths = self_read.built_paths.as_ref();
        let first_built_path = built_paths.and_then(|bp| bp.first())?;
        let main_program_meta_path = self_read.attr.child("meta".to_owned()).child("mainProgram".to_owned());
        let main_program_meta = self_read.file.output(main_program_meta_path, &[]).ok();
        main_program_meta
            .and_then(|mp| mp.eval(&["--raw".to_string()]).ok())
            .map(|mp| first_built_path.join("bin").join(mp))
            .or_else(|| self.drv_name().ok().map(|n| first_built_path.join("bin").join(n)))
            .ok_or_else(|| NieError::ProgramNotFound(self.reference().into()))
            .ok()
    }

    pub fn reference(&self) -> NixReference {
        self.file().reference().with_attribute(self.attr())
    }

    pub fn build(&self, allow_out_links: bool, extra_args: &[String], remote: Option<&str>) -> NieResult<Vec<PathBuf>> {
        let attr = self.attr().clone();
        let path = self.file().path();

        if let Some(paths) = &self.0.read().unwrap().built_paths {
            return Ok(paths.clone())
        }

        inform(&format!("Building {} from {}{}{}",
                attr.to_string_user(),
                path.to_string_lossy(),
                if self.file().flake_compat() { " with flake_compat" } else { "" },
                if let Some(remote) = remote { format!(" on {}", remote) } else { String::new() }));

        let paths = if let Some(remote) = remote {
            let checkout = self.file().checkout();
            let inputs = vec!(checkout.path().as_path());
            nix::build_remote(&inputs, &path, &attr, remote, &self.file().eval_args(), extra_args)?
        } else {
            nix::build(&path, &attr, allow_out_links, &self.file().eval_args(), extra_args)?
        };

        self.0.write().unwrap().built_paths = Some(paths.clone());
        Ok(paths)
    }

    pub fn eval(&self, extra_args: &[String]) -> NieResult<String> {
        let attr = self.attr().clone();
        let path = self.file().path();

        if self.file().flake_compat() {
            inform(&format!("Evaluating {} from {} with flake-compat", attr.to_string_user(), path.to_string_lossy()));
        } else {
            inform(&format!("Evaluating {} from {}", attr.to_string_user(), path.to_string_lossy()));
        };

        let output = nix::eval(&path, &attr, &self.file().eval_args(), extra_args)?;
        Ok(output)
    }

    pub fn enter_dev_shell(&self, command: Option<String>, extra_args: &[String]) -> NieResult<()> {
        let attr = self.attr().clone();
        let path = self.file().path();

        inform(&format!("Creating dev shell {} from {}", attr.to_string_user(), path.to_string_lossy()));
        nix::dev_shell(&path, &attr, &self.file().eval_args(), command, extra_args)
    }

    pub fn create_drv_gc_root(&self, root: &Path) -> NieResult<()> {
        let attr = self.attr().clone();
        let path = self.file().path();

        inform(&format!("Creating gc root {} for derivation of {}", root.to_string_lossy(), attr));
        nix::create_root(&path, &attr, &self.file().eval_args(), root).map(|_| ())
    }

}
