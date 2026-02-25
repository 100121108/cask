use std::fs;

use anyhow::{Context, Result, bail};
use nix::sys::signal;
use nix::unistd::Pid;

use crate::cli::DataDirOpt;

pub fn execute(opts: DataDirOpt) -> Result<()> {
    let pid_path = opts.data_dir.join("cask.pid");
    if !pid_path.exists() {
        bail!("no PID file found — is cask running?");
    }

    let contents = fs::read_to_string(&pid_path).context("failed to read PID file")?;
    let pid: i32 = contents
        .trim()
        .parse()
        .context("invalid PID in PID file")?;

    // Verify the process is alive
    if signal::kill(Pid::from_raw(pid), None).is_err() {
        eprintln!("Stale PID file (process {} is not running).", pid);
        let _ = fs::remove_file(&pid_path);
        bail!("cask is not running");
    }

    println!("{}", pid);
    Ok(())
}
