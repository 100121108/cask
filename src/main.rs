mod auth;
mod cli;
mod commands;
mod db;
mod error;
mod server;
mod state;
mod storage;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Start(opts) => commands::start::execute(opts),
        Command::Run(opts) => commands::run::execute(opts),
        Command::Stop(opts) => commands::stop::execute(opts),
        Command::Pid(opts) => commands::pid::execute(opts),
        Command::Log(opts) => commands::log::execute(opts),
    }
}
