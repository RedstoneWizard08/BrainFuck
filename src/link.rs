//! AOT (ahead-of-time) linking for compiled object files.
//!
//! This module handles the final linking stage for compiled Brainf*ck programs,
//! converting object files into executable binaries.
//!
//! # Examples
//!
//! Linking a compiled object file:
//!
//! ```no_run
//! use bf::link::link_aot;
//! use std::path::PathBuf;
//! use target_lexicon::Triple;
//!
//! let obj_data = vec![/* ... object file bytes ... */];
//! let output = PathBuf::from("./program");
//! let target = Triple::host();
//! link_aot(obj_data, output, &target);
//! ```

use crate::linker::run_linker;
use std::{io::Write, path::PathBuf};
use target_lexicon::Triple;
use tempfile::NamedTempFile;

/// Links a compiled object file into an executable binary.
///
/// # Arguments
///
/// * `obj` - The object file data as a byte vector
/// * `out` - The output path for the final executable
/// * `target` - The target triple for the compilation
pub fn link_aot(obj: Vec<u8>, out: PathBuf, target: &Triple) {
    let mut temp = NamedTempFile::with_suffix(".o").unwrap();

    temp.write_all(&obj).unwrap();

    let result = run_linker(
        out,
        None,
        temp.path().into(),
        target.clone(),
        vec![],
        vec![],
    )
    .is_ok();

    temp.close().unwrap();

    if !result {
        panic!("Failed to link binary file!");
    }
}
