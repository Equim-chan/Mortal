use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio, exit};

use anyhow::{Context, Result, bail};

// All relative to $EXE_DIR
const CONDA_ENV: &str = "env";
const TARGET: &str = "mortal.py";

const SEP: &str = ";";

macro_rules! canonicalize {
    ($path:ident) => {{
        let p = if $path.as_os_str().is_empty() {
            Path::new(".")
        } else {
            $path.as_ref()
        };
        dunce::canonicalize(p).with_context(|| {
            format!(
                "failed to canonicalize {}: \"{}\" (does it exist?)",
                stringify!($path),
                $path.display(),
            )
        })
    }};
}

fn main() -> Result<()> {
    // This crate is designed for windows only, but CI runners are in linux.
    #[cfg(not(target_os = "windows"))]
    eprintln!("This tool is for Windows only.");

    let exe = env::current_exe()?;
    let exe_dir = exe.parent().context("no parent")?;
    let exe_dir = canonicalize!(exe_dir)?;
    let env = exe_dir.join(CONDA_ENV);

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
        .arg(TARGET)
        .args(&args[1..])
        .env("PATH", path_str)
        .current_dir(exe_dir)
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
