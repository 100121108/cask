use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::cli::LogOpts;

pub fn execute(opts: LogOpts) -> Result<()> {
    let log_path = opts.data_dir.join("cask.log");
    if !log_path.exists() {
        bail!("no log file found at {}", log_path.display());
    }

    let file = File::open(&log_path)
        .with_context(|| format!("failed to open {}", log_path.display()))?;
    let reader = BufReader::new(&file);

    // Read all lines, then take last N
    let lines: Vec<String> = reader.lines().collect::<io::Result<Vec<_>>>()?;
    let start = lines.len().saturating_sub(opts.n);
    for line in &lines[start..] {
        println!("{}", line);
    }

    if opts.f {
        // Seek to end and follow
        let mut file = File::open(&log_path)?;
        file.seek(SeekFrom::End(0))?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    thread::sleep(Duration::from_millis(100));
                }
                Ok(_) => {
                    print!("{}", line);
                }
                Err(e) => {
                    bail!("error reading log: {}", e);
                }
            }
        }
    }

    Ok(())
}
