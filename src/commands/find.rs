use crate::archive::ZipManager;
use crate::errors::ZipCrawlError;
use regex::Regex;

use glob::Pattern;

pub fn handle(
    manager: &mut ZipManager,
    query: &str,
    path_filter: Option<&str>,
    use_glob: bool,
    entry_type: Option<&str>,
) -> Result<(), ZipCrawlError> {
    let filter_f = entry_type == Some("f");
    let filter_d = entry_type == Some("d");

    for entry in manager.entries()? {
        if filter_f && entry.is_dir {
            continue;
        }
        if filter_d && !entry.is_dir {
            continue;
        }

        if let Some(ref p) = path_filter
            && !entry.name.starts_with(p)
        {
            continue;
        }

        let is_match = if use_glob {
            Pattern::new(query)
                .map(|p| p.matches(&entry.name))
                .unwrap_or(false)
        } else {
            Regex::new(query)
                .map(|re| re.is_match(&entry.name))
                .unwrap_or(false)
        };

        if is_match {
            println!("{}", entry.name);
        }
    }
    Ok(())
}
