use std::iter;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::error::{NieError, NieResult};
use crate::interact::*;
use crate::location::{AttributePath, NixReference};
use crate::registry::Registry;
use crate::store::checkout::Checkout;
use crate::store::file::NixFile;
use crate::{EvalArgs, nix};


/// Registry to cache known [`NixOutput`]s
static OUTPUT_REGISTRY: Registry<NixReference, NixOutput> = Registry::new();

/// An output path of a checked-out, locally available .nix file.
///
/// Derived from a [`NixFile`] (see [`NixFile::output()`] and [`NixFile::outputs()`]) and an
/// attribute path as described by a [`NixReference`].
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

    /// Fetch and build all outputs described by `refs`.
    ///
    /// This combines calls to [`Checkout::fetch_all()`], [`Checkout::files()`],
    /// [`NixFile::outputs()`] and [`NixOutput::build()`].
    pub fn fetch_and_build_all(refs: &[NixReference], common_locations: &[AttributePath], allow_out_links: bool,
            eval_args: &EvalArgs, extra_args: &[String], remote: Option<&str>) -> NieResult<Vec<Vec<PathBuf>>> {
        let repo_refs = refs.iter().map(|s| s.repository()).cloned();
        let filenames = refs.iter().map(|s| s.filename().cloned());
        let attributes = refs.iter().map(|s| s.attribute()).cloned();

        let checkouts = Checkout::fetch_all(repo_refs)?;
        let files = Checkout::files(iter::zip(checkouts.iter().cloned(), filenames), eval_args.clone())?;
        let outputs = NixFile::outputs(iter::zip(files.iter().cloned(), attributes), common_locations)?;

        let mut built = Vec::new();
        for (i, output) in outputs.into_iter().enumerate() {
            let rename = match i {
                0 => Some(format!("result-{}", i)),
                _ => Some("result".to_owned()),
            };

            let out_path = output.build(rename.as_deref(), allow_out_links, extra_args, remote)
                .and_then(|p| if p.is_empty() {
                    Err(NieError::NoOutputPath(output.reference().into()))
                } else { Ok(p) })?;
            built.push(out_path);
        }

        Ok(built)
    }

    pub fn file(&self) -> NixFile {
        self.0.read().unwrap().file.clone()
    }

    pub fn attr(&self) -> AttributePath {
        self.0.read().unwrap().attr.clone()
    }

    /// Return the "drv name" as described by the
    /// [Nix manual](https://nix.dev/manual/nix/stable/)
    /// entry for the builtin
    /// [`parseDrvName`](https://nix.dev/manual/nix/stable/language/builtins.html#builtins-parseDrvName).
    pub fn drv_name(&self) -> NieResult<String> {
        let paths = self.build(None, false, &[], None)?;
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

    /// Return the path of the main program of the output by querying the `meta.mainProgram` field
    /// of the derivation and falling back to a binary name based on the derivations name if it is
    /// not specified.
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

    /// Return the path of the main program's man page based on [`NixOutput::main_program()`]
    pub fn man_path(&self) -> Option<PathBuf> {
        let self_read = self.0.read().unwrap();
        let built_paths = self_read.built_paths.as_ref();
        let first_built_path = built_paths.and_then(|bp| bp.first())?;
        let name = self.main_program().map(|p| p.to_string_lossy().to_string())?;

        let man_path = first_built_path.join("share")
            .join("man")
            .join("man1")
            .join(format!("{}.1.gz", name));

        Some(man_path)
    }

    /// Create a reference from the current output
    pub fn reference(&self) -> NixReference {
        self.file().reference().with_attribute(self.attr())
    }

    /// Build the output
    ///
    /// - `rename` can specify a name for the output link
    /// - `allow_out_links` decides if a output link will be created at all
    /// - `extra_args` may contain additional args passed to `nix-build`.
    /// - `remote` may specify a host for remote building via ssh.
    pub fn build(&self, rename: Option<&str>, allow_out_links: bool, extra_args: &[String],
            remote: Option<&str>) -> NieResult<Vec<PathBuf>> {
        let attr = self.attr().clone();
        let path = self.file().path();

        if let Some(paths) = &self.0.read().unwrap().built_paths {
            return Ok(paths.clone())
        }

        let extra_args: Vec<_> = if let Some(name) = rename {
            let mut v = vec!["-o", name];
            v.extend(extra_args.iter().map(|s| s.as_str()));
            v
        } else {
            extra_args.iter().map(|s| s.as_str()).collect()
        };

        inform_build(&attr, &self.file(), self.file().eval_args().flake_compat, remote);

        let paths = if let Some(remote) = remote {
            let checkout = self.file().checkout();
            let inputs = vec!(checkout.path().as_path());
            nix::build_remote(&inputs, &path, &attr, remote, &self.file().eval_args(), &extra_args)?
        } else {
            nix::build(&path, &attr, allow_out_links, &self.file().eval_args(), &extra_args)?
        };

        self.0.write().unwrap().built_paths = Some(paths.clone());
        Ok(paths)
    }

    /// Evaluate the attribute
    pub fn eval(&self, extra_args: &[String]) -> NieResult<String> {
        let attr = self.attr().clone();
        let path = self.file().path();

        inform_eval(&attr, &self.file(), self.file().eval_args().flake_compat);

        let output = nix::eval(&path, &attr, &self.file().eval_args(), extra_args)?;
        Ok(output)
    }

    /// Enter a development shell derived from the output
    pub fn enter_dev_shell(&self, command: Option<String>, extra_args: &[String]) -> NieResult<()> {
        let attr = self.attr().clone();
        let path = self.file().path();

        inform_enter_dev_shell(&attr, &self.file());
        nix::dev_shell(&path, &attr, &self.file().eval_args(), command, extra_args)
    }

    /// Create a named gc root from the output
    pub fn create_drv_gc_root(&self, root: &Path) -> NieResult<()> {
        let attr = self.attr().clone();
        let path = self.file().path();

        inform_create_gc_root(root, &attr, &self.file());
        nix::create_root(&path, &attr, &self.file().eval_args(), root).map(|_| ())
    }
}
