use crate::archive::ZipManager;
use crate::display::TreeWriter;
use crate::errors::ZipCrawlError;
use glob::Pattern;
use std::io;

pub fn handle(
    manager: &mut ZipManager,
    file_pattern: &str,
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

        let mut entry = manager.open_file(&file_name)?;
        io::copy(&mut entry, &mut io::stdout()).map_err(|e| ZipCrawlError::IoError {
            path: file_name,
            source: e,
        })?;

        if !quiet {
            println!();
        }
    }

    Ok(())
}
