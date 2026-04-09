use glob::Pattern;

use crate::archive::ZipManager;
use crate::display::TreeWriter;
use crate::errors::ZipCrawlError;
use std::io;
use std::process::{Command, Stdio};

pub fn handle(
    manager: &mut ZipManager,
    file_pattern: &str,
    command: &str,
    args: &[String],
    quiet: bool,
) -> Result<(), ZipCrawlError> {
    let pattern = Pattern::new(file_pattern).map_err(|_| ZipCrawlError::InvalidPath {
        path: file_pattern.to_string(),
    })?;

    let entries = manager.entries()?;
    let matches: Vec<String> = entries
        .iter()
        .filter(|e| !e.is_dir && pattern.matches(&e.name))
        .map(|e| e.name.clone())
        .collect();

    if matches.is_empty() {
        return Err(ZipCrawlError::FileNotFound {
            filename: file_pattern.to_string(),
        });
    }

    for file_name in matches {
        if !quiet {
            TreeWriter::print_file_header(&file_name);
        }

        manager.stream_file(&file_name, |reader| {
            let mut child = Command::new(command)
                .args(args)
                .stdin(Stdio::piped())
                .spawn()
                .map_err(|e| ZipCrawlError::ExecutionError {
                    cmd: command.to_string(),
                    source: e,
                })?;

            if let Some(mut stdin) = child.stdin.take() {
                io::copy(reader, &mut stdin).map_err(|e| ZipCrawlError::ExecutionError {
                    cmd: command.to_string(),
                    source: e,
                })?;
            }

            let status = child.wait().map_err(|e| ZipCrawlError::ExecutionError {
                cmd: command.to_string(),
                source: e,
            })?;

            if !status.success() {
                eprintln!("Warning: Command failed for {file_name}");
            }
            Ok(())
        })?;

        if !quiet {
            println!();
        }
    }

    Ok(())
}
