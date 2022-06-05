use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process::{exit, Command, Stdio};

use anyhow::{bail, Context, Result};

const CONDA_ENV: &str = "env";
const TARGET: &str = "mortal.py";

// This crate is designed for windows only, but CI runners are in linux, so I'll
// just let it pass.
#[cfg(target_os = "windows")]
const SEP: &str = ";";
#[cfg(not(target_os = "windows"))]
const SEP: &str = ":";

macro_rules! canonicalize {
    ($path:ident) => {
        let p = if $path.as_os_str().is_empty() {
            Path::new(".")
        } else {
            $path.as_ref()
        };
        #[allow(unused_variables)]
        let $path = dunce::canonicalize(p).with_context(|| {
            format!(
                "failed to canonicalize {}: \"{}\" (does it exist?)",
                stringify!($path),
                $path.display(),
            )
        })?;
    };
}

fn main() -> Result<()> {
    let exe = env::current_exe()?;
    let exe_dir = exe.parent().context("no parent")?;
    canonicalize!(exe_dir);
    let env = exe_dir.join(CONDA_ENV);

    let target_path = Path::new(TARGET);
    let target = target_path.file_name().context("no file name")?;
    let target_dir = target_path.parent().context("no parent")?;
    canonicalize!(target_dir);

    let mut paths = vec![
        env.join("bin").into_os_string(),
        env.join("Scripts").into_os_string(),
        env.join("Library/bin").into_os_string(),
        env.join("Library/usr/bin").into_os_string(),
        env.join("Library/mingw-w64/bin").into_os_string(),
        env.into_os_string(),
    ];

    let path_env = env::var_os("PATH").unwrap_or_default();
    if !path_env.is_empty() {
        paths.push(path_env);
    }

    let mut path_str = OsString::new();
    for (i, item) in paths.into_iter().enumerate() {
        if i > 0 {
            path_str.push(SEP);
        }
        path_str.push(item);
    }

    let args: Vec<_> = env::args_os().collect();
    let mut proc = Command::new("python")
        .arg(target)
        .args(&args[1..])
        .env("PATH", path_str)
        .current_dir(target_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let status = proc.wait()?;
    if !status.success() {
        if let Some(code) = status.code() {
            exit(code);
        }
        bail!("process terminated by signal");
    }

    Ok(())
}
