use std::fs;

use anyhow::{Context, Result, bail};
use daemonize::Daemonize;

use crate::cli::ServerOpts;
use crate::server;

pub fn execute(opts: ServerOpts) -> Result<()> {
    let data_dir = &opts.data_dir;
    fs::create_dir_all(data_dir)
        .with_context(|| format!("failed to create data dir: {}", data_dir.display()))?;

    let pid_path = data_dir.join("cask.pid");
    if pid_path.exists() {
        let contents = fs::read_to_string(&pid_path).unwrap_or_default();
        if let Ok(pid) = contents.trim().parse::<i32>() {
            use nix::sys::signal::kill;
            use nix::unistd::Pid;
            if kill(Pid::from_raw(pid), None).is_ok() {
                bail!(
                    "cask is already running (PID {}). Use `cask stop` first.",
                    pid
                );
            }
        }
        // Stale PID file — remove it
        let _ = fs::remove_file(&pid_path);
    }

    let log_path = data_dir.join("cask.log");
    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("failed to open log file: {}", log_path.display()))?;

    let stdout = log_file
        .try_clone()
        .context("failed to clone log file handle")?;
    let stderr = log_file
        .try_clone()
        .context("failed to clone log file handle")?;

    let daemonize = Daemonize::new()
        .pid_file(&pid_path)
        .stdout(stdout)
        .stderr(stderr)
        .working_directory(".");

    eprintln!(
        "Starting cask daemon on {}:{}...",
        opts.host, opts.port
    );

    daemonize.start().context("failed to daemonize")?;

    // We are now the child process
    server::run(opts, false)
}
