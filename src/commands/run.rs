use anyhow::Result;

use crate::cli::ServerOpts;
use crate::server;

pub fn execute(opts: ServerOpts) -> Result<()> {
    server::run(opts, true)
}
