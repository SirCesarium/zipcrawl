use crate::archive::{ZipEntry, ZipManager};
use crate::display::{DiffWriter, TreeWriter};
use crate::errors::ZipCrawlError;
use glob::Pattern;
use minus::Pager;
use similar::TextDiff;
use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;
use std::io;
use std::path::Path;

#[allow(
    clippy::too_many_lines,
    clippy::fn_params_excessive_bools,
    clippy::too_many_arguments
)]
pub fn handle(
    manager: &mut ZipManager,
    base_path: &str,
    show_struct: bool,
    show_sizes: bool,
    show_content: bool,
    full_content: bool,
    include: Option<&[String]>,
    exclude: Option<&[String]>,
    quiet: bool,
) -> Result<(), ZipCrawlError> {
    let mut base_manager = ZipManager::new(Path::new(base_path))?;
    let mut output = String::new();

    let include_patterns: Vec<Pattern> = include
        .unwrap_or_default()
        .iter()
        .filter_map(|p| Pattern::new(p).ok())
        .collect();

    let exclude_patterns: Vec<Pattern> = exclude
        .unwrap_or_default()
        .iter()
        .filter_map(|p| Pattern::new(p).ok())
        .collect();

    let current_map: HashMap<String, ZipEntry> = manager
        .entries()?
        .into_iter()
        .map(|e| (e.name.clone(), e))
        .collect();

    let base_map: HashMap<String, ZipEntry> = base_manager
        .entries()?
        .into_iter()
        .map(|e| (e.name.clone(), e))
        .collect();

    let all_paths: BTreeSet<_> = current_map.keys().chain(base_map.keys()).collect();
    let default_mode = !show_struct && !show_sizes && !show_content && !full_content;

    for path in &all_paths {
        if !include_patterns.is_empty() && !include_patterns.iter().any(|p| p.matches(path)) {
            continue;
        }
        if exclude_patterns.iter().any(|p| p.matches(path)) {
            continue;
        }

        let current = current_map.get(*path);
        let base = base_map.get(*path);

        match (current, base) {
            (Some(c), None) if show_struct || default_mode => {
                writeln!(output, "{}", DiffWriter::format_addition(path, c.is_dir)).ok();
            }
            (None, Some(b)) if show_struct || default_mode => {
                writeln!(output, "{}", DiffWriter::format_removal(path, b.is_dir)).ok();
            }
            (Some(c), Some(b)) if !c.is_dir => {
                let mut diffs = Vec::new();
                let content_changed = c.crc != b.crc;

                if show_sizes && c.size != b.size {
                    diffs.push(format!(
                        "size: {} ➜ {}",
                        TreeWriter::format_size(b.size),
                        TreeWriter::format_size(c.size)
                    ));
                }

                if (show_content || default_mode) && content_changed {
                    diffs.push("content changed".to_string());
                }

                if !diffs.is_empty() {
                    writeln!(
                        output,
                        "{}",
                        DiffWriter::format_change(path, c.is_dir, &diffs)
                    )
                    .ok();
                }

                if full_content && content_changed {
                    let extension = Path::new(path)
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");

                    let text_extensions = [
                        "toml",
                        "json",
                        "txt",
                        "js",
                        "properties",
                        "fmdata",
                        "db",
                        "local",
                        "mcmeta",
                    ];

                    if text_extensions.contains(&extension) && c.size < 200_000 && b.size < 200_000
                    {
                        let data_current = manager.read_file_content(&c.name)?;
                        let data_base = base_manager.read_file_content(&b.name)?;

                        let text_current = String::from_utf8_lossy(&data_current);
                        let text_base = String::from_utf8_lossy(&data_base);

                        let diff = TextDiff::from_lines(&text_base, &text_current);

                        writeln!(output, "{}", DiffWriter::line_diff_header()).ok();
                        for change in diff.iter_all_changes() {
                            write!(
                                output,
                                "{}",
                                DiffWriter::format_line_diff(change.tag(), &change.to_string())
                            )
                            .ok();
                        }
                        writeln!(output, "{}", DiffWriter::line_diff_footer()).ok();
                    }
                }
            }
            _ => {}
        }
    }

    if quiet {
        if !output.is_empty() {
            println!("{output}");
        }
        return Ok(());
    }

    let pager = Pager::new();
    pager.set_text(output).map_err(|e| ZipCrawlError::IoError {
        path: "pager_text".to_string(),
        source: io::Error::other(e),
    })?;

    pager
        .set_prompt("zipcrawl diff")
        .map_err(|e| ZipCrawlError::IoError {
            path: "pager_prompt".to_string(),
            source: io::Error::other(e),
        })?;

    minus::page_all(pager).map_err(|e| ZipCrawlError::IoError {
        path: "pager_output".to_string(),
        source: io::Error::other(e),
    })?;

    Ok(())
}
