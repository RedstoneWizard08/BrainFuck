use crate::linker::command_exists;
use anyhow::Result;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};
use target_lexicon::Triple;

pub fn get_default_linker() -> &'static str {
    for cmd in &["mold", "ld"] {
        if command_exists(cmd) {
            return cmd;
        }
    }

    panic!("Could not find a linker!");
}

pub fn get_linker_args(
    _prefix_dir: Option<String>,
    _target_arch: Option<String>,
    _target_os: Option<String>,
    _target_env: Option<String>,
) -> Vec<String> {
    use std::io::Read;

    let mut s = String::new();

    std::process::Command::new("xcrun")
        .arg("-sdk")
        .arg("macosx")
        .arg("--show-sdk-path")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap()
        .stdout
        .unwrap()
        .read_to_string(&mut s)
        .unwrap();

    vec![
        "-arch".to_string(),
        "arm64".to_string(),
        "-platform_version".to_string(),
        "macos".to_string(),
        "14.0".to_string(),
        "15.5".to_string(),
        "-lc".to_string(),
        "-syslibroot".to_string(),
        s,
    ]
}

pub fn run_linker(
    out_path: PathBuf,
    linker: Option<String>,
    tmp_file: PathBuf,
    triple: Triple,
    mut extra_args: Vec<String>,
    extra_libs: Vec<String>,
) -> Result<()> {
    extra_args.extend(extra_libs.iter().map(|v| format!("-l{}", v)));

    let linker = linker.unwrap_or(get_default_linker().to_string());

    // using super:: here allows android to work with its custom library dir
    let args = super::get_linker_args(
        None,
        Some(triple.architecture.to_string()),
        Some(triple.operating_system.to_string()),
        Some(triple.environment.to_string()),
    );

    let cmd_str = format!(
        "{} -o {} {} {} {}",
        linker,
        out_path.to_str().unwrap(),
        args.join(" "),
        extra_args.join(" "),
        tmp_file.to_str().unwrap()
    );

    log::debug!("Running linker with command: {}", cmd_str);

    Command::new(linker)
        .arg("-o")
        .arg(out_path)
        .args(args)
        .args(extra_args)
        .arg(tmp_file)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?
        .wait()?;

    Ok(())
}
