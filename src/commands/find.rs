use crate::archive::ZipManager;
use crate::errors::ZipCrawlError;
use regex::Regex;

pub fn handle(manager: &mut ZipManager, regex: &str) -> Result<(), ZipCrawlError> {
    let re = Regex::new(regex).map_err(|e| ZipCrawlError::InvalidRegex {
        regex: regex.to_string(),
        source: e,
    })?;
    for entry in manager.entries()? {
        if re.is_match(&entry.name) {
            println!("[{}] {}", manager.path_name, entry.name);
        }
    }
    Ok(())
}
