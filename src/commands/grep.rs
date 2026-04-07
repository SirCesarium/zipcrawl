use crate::archive::ZipManager;
use crate::display::TreeWriter;
use crate::errors::ZipCrawlError;
use colored::Colorize;
use regex::Regex;
use std::io::{BufRead, BufReader};

pub fn handle(manager: &mut ZipManager, pattern: &str) -> Result<(), ZipCrawlError> {
    let re = Regex::new(pattern).map_err(|e| ZipCrawlError::InvalidRegex {
        regex: pattern.to_string(),
        source: e,
    })?;

    let entries = manager.entries()?;
    for entry in entries {
        if !entry.is_dir {
            let mut file = manager.open_file(&entry.name)?;
            let reader = BufReader::new(&mut file);
            let mut matched_lines = Vec::new();

            for line_result in reader.lines() {
                if let Ok(line) = line_result
                    && re.is_match(&line)
                {
                    let highlighted = re
                        .replace_all(&line, |caps: &regex::Captures| {
                            caps[0].red().bold().to_string()
                        })
                        .into_owned();
                    matched_lines.push(highlighted);
                }
            }

            if !matched_lines.is_empty() {
                let icon = TreeWriter::get_icon_for_name(&entry.name, false);
                println!("{} {}", icon, entry.name.cyan().bold());
                for line in matched_lines {
                    println!("  {}", line.trim());
                }
                println!();
            }
        }
    }
    Ok(())
}
