use crate::archive::ZipManager;
use crate::errors::ZipCrawlError;
use std::io;
use std::process::{Command, Stdio};

pub fn handle(
    manager: &mut ZipManager,
    file: &str,
    command: &str,
    args: &[String],
) -> Result<(), ZipCrawlError> {
    let mut entry = manager.open_file(file)?;
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| ZipCrawlError::ExecutionError {
            cmd: command.to_string(),
            source: e,
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = io::copy(&mut entry, &mut stdin);
    }
    let _ = child.wait();
    Ok(())
}
