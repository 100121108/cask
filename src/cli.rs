use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cask", about = "Lightweight artifact hosting server")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the server as a background daemon
    Start(ServerOpts),

    /// Run the server in the foreground
    Run(ServerOpts),

    /// Stop a running daemon
    Stop(DataDirOpt),

    /// Print the PID of a running daemon
    Pid(DataDirOpt),

    /// Show recent log output
    Log(LogOpts),
}

#[derive(Parser, Clone)]
pub struct ServerOpts {
    /// Address to bind to
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Port to listen on
    #[arg(long, default_value_t = 8080)]
    pub port: u16,

    /// Directory for database, logs, PID file, and artifacts
    #[arg(long, default_value = "./data")]
    pub data_dir: PathBuf,

    /// Maximum upload size in bytes
    #[arg(long, default_value_t = 100 * 1024 * 1024)]
    pub max_upload_size: usize,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    pub log_level: String,
}

#[derive(Parser, Clone)]
pub struct DataDirOpt {
    /// Directory for database, logs, PID file, and artifacts
    #[arg(long, default_value = "./data")]
    pub data_dir: PathBuf,
}

#[derive(Parser, Clone)]
pub struct LogOpts {
    /// Directory for database, logs, PID file, and artifacts
    #[arg(long, default_value = "./data")]
    pub data_dir: PathBuf,

    /// Number of lines to show
    #[arg(short, default_value_t = 20)]
    pub n: usize,

    /// Follow the log (like tail -f)
    #[arg(short)]
    pub f: bool,
}
