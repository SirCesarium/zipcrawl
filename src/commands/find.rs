use crate::archive::ZipManager;
use crate::errors::ZipCrawlError;
use glob::Pattern;
use regex::Regex;

pub fn handle(
    manager: &mut ZipManager,
    query: &str,
    path_filter: Option<&str>,
    use_glob: bool,
    entry_type: Option<&str>,
) -> Result<(), ZipCrawlError> {
    let filter_f = entry_type == Some("f");
    let filter_d = entry_type == Some("d");

    let glob_matcher = if use_glob {
        Some(Pattern::new(query).map_err(|_| ZipCrawlError::InvalidGlob {
            glob: query.to_string(),
        })?)
    } else {
        None
    };

    let regex_matcher = if !use_glob {
        Some(Regex::new(query).map_err(|e| ZipCrawlError::InvalidRegex {
            regex: query.to_string(),
            source: e,
        })?)
    } else {
        None
    };

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

        let is_match = if let Some(ref p) = glob_matcher {
            p.matches(&entry.name)
        } else if let Some(ref re) = regex_matcher {
            re.is_match(&entry.name)
        } else {
            false
        };

        if is_match {
            println!("{}", entry.name);
        }
    }
    Ok(())
}
