use anyhow::Result;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};
use target_lexicon::{Environment, Triple};

use crate::linker::command_exists;

pub fn get_default_linker() -> &'static str {
    for cmd in &["mold", "ld.lld", "ld.gold", "ld"] {
        if command_exists(cmd) {
            return cmd;
        }
    }

    panic!("Could not find a linker!");
}

pub fn get_dynamic_linker(
    prefix: String,
    target_arch: Option<String>,
    target_env: Option<String>,
) -> String {
    let env = target_env.unwrap_or(Environment::host().to_string());

    if env == "android" {
        return "/system/bin/linker64".into();
    }

    if target_arch == Some("x86_64".into()) {
        return format!("{}/lib64/ld-linux-x86-64.so.2", prefix);
    }

    #[cfg(target_arch = "x86_64")]
    {
        format!("{}/lib64/ld-linux-x86-64.so.2", prefix)
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        format!(
            "{}/lib/ld-linux-{}.so.1",
            prefix,
            target_arch.unwrap_or(std::env::consts::ARCH.into())
        )
    }
}

#[cfg(not(target_os = "android"))]
pub fn get_linker_args(
    prefix_dir: Option<String>,
    target_arch: Option<String>,
    target_os: Option<String>,
    target_env: Option<String>,
) -> Vec<String> {
    use std::env::consts::{ARCH, OS};

    let env = Environment::host().to_string();
    let prefix = prefix_dir.unwrap_or(std::env::var("PREFIX").unwrap_or(String::from("/usr")));

    vec![
        format!("-L{}/lib", prefix),
        format!(
            "-L{}/lib/{}-{}-{}",
            prefix,
            target_arch.clone().unwrap_or(ARCH.into()),
            target_os.clone().unwrap_or(OS.into()),
            target_env.clone().unwrap_or(env.clone())
        ),
        format!(
            "-L{}/{}-{}-{}/lib",
            prefix,
            target_arch.clone().unwrap_or(ARCH.into()),
            target_os.unwrap_or(OS.into()),
            target_env.clone().unwrap_or(env)
        ),
        "--dynamic-linker".into(),
        get_dynamic_linker(prefix, target_arch, target_env),
        "--pie".into(),
        "-O2".into(),
        "-lc".into(),
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
