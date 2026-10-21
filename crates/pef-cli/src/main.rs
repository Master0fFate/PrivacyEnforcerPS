//! `pef`: audit-first Windows privacy policies from the terminal.
//!
//! Audit is the default. Launching creates no logs, directories, elevation prompts or
//! background services.

#![forbid(unsafe_code)]

use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut host = pef_cli::SystemHost;
    let stdout = io::stdout();
    let mut out = stdout.lock();
    match pef_cli::run(&args, &mut host, &mut out) {
        Ok(code) => {
            let _ = out.flush();
            ExitCode::from(code)
        }
        Err(failure) => {
            let _ = out.flush();
            let _ = writeln!(io::stderr(), "{}", failure.render());
            ExitCode::from(pef_cli::EXIT_STOPPED)
        }
    }
}
