use crate::archive::ZipManager;
use crate::errors::ZipCrawlError;
use std::io;

pub fn handle(manager: &mut ZipManager, file: &str) -> Result<(), ZipCrawlError> {
    let mut entry = manager.open_file(file)?;
    io::copy(&mut entry, &mut io::stdout())
        .map(|_| ())
        .map_err(|e| ZipCrawlError::IoError {
            path: file.to_string(),
            source: e,
        })
}
