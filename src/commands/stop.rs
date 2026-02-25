use std::fs;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use nix::sys::signal::{self, Signal};
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
    let nix_pid = Pid::from_raw(pid);

    // Check if process is alive
    if signal::kill(nix_pid, None).is_err() {
        eprintln!("Process {} is not running. Cleaning up stale PID file.", pid);
        let _ = fs::remove_file(&pid_path);
        return Ok(());
    }

    eprintln!("Sending SIGTERM to cask (PID {})...", pid);
    signal::kill(nix_pid, Signal::SIGTERM).context("failed to send SIGTERM")?;

    // Poll until dead (10s timeout)
    for _ in 0..100 {
        thread::sleep(Duration::from_millis(100));
        if signal::kill(nix_pid, None).is_err() {
            eprintln!("cask stopped.");
            let _ = fs::remove_file(&pid_path);
            return Ok(());
        }
    }

    bail!("cask (PID {}) did not stop within 10 seconds", pid);
}
