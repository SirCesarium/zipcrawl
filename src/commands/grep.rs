use crate::archive::ZipManager;
use crate::display::TreeWriter;
use crate::errors::ZipCrawlError;
use colored::Colorize;
use regex::Regex;
use std::io::{BufRead, BufReader};

pub fn handle(
    manager: &mut ZipManager,
    pattern: &str,
    glob_filter: Option<&str>,
    path_filter: Option<&str>,
) -> Result<(), ZipCrawlError> {
    let re = Regex::new(pattern).map_err(|e| ZipCrawlError::InvalidRegex {
        regex: pattern.to_string(),
        source: e,
    })?;

    let glob_matcher = glob_filter
        .map(|g| {
            glob::Pattern::new(g).map_err(|_| ZipCrawlError::InvalidGlob {
                glob: g.to_string(),
            })
        })
        .transpose()?;

    let archive_path = manager.path_name.dimmed();
    let entries = manager.entries()?;

    for entry in entries {
        if entry.is_dir {
            continue;
        }

        if path_filter.is_some_and(|p| !entry.name.starts_with(p))
            || glob_matcher
                .as_ref()
                .is_some_and(|m| !m.matches(&entry.name))
        {
            continue;
        }

        manager.stream_file(&entry.name, |file| {
            let reader = BufReader::new(file);
            let mut matched = false;

            for (idx, line_result) in reader.lines().enumerate() {
                let Ok(line) = line_result else { break };

                if re.is_match(&line) {
                    if !matched {
                        let icon = TreeWriter::get_icon_for_name(&entry.name, false);
                        println!(
                            "{} {} {} {}",
                            icon,
                            archive_path,
                            "➜".bright_black(),
                            entry.name.cyan().bold()
                        );
                        matched = true;
                    }

                    let line_num = (idx + 1).to_string().dimmed();
                    let highlighted = re.replace_all(&line, |caps: &regex::Captures| {
                        caps[0].red().bold().to_string()
                    });

                    println!("  {} {}", line_num, highlighted.trim());
                }
            }

            if matched {
                println!();
            }
            Ok(())
        })?;
    }
    Ok(())
}
