use clap::CommandFactory;
use clap_complete::{generate, shells::Shell};
use std::io;

use crate::commands::Cli;

pub fn handle(shell: Shell) -> Result<(), io::Error> {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "zipcrawl", &mut io::stdout());
    Ok(())
}
