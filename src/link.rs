use crate::linker::run_linker;
use std::{io::Write, path::PathBuf};
use target_lexicon::Triple;
use tempfile::NamedTempFile;

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
