use crate::archive::ZipManager;
use crate::display::TreeWriter;
use crate::errors::ZipCrawlError;
use colored::Colorize;
use glob::Pattern;
use minus::Pager;
use std::io::{self, Read};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;
use syntect::util::LinesWithEndings;

pub fn handle(manager: &mut ZipManager, file_pattern: &str) -> Result<(), ZipCrawlError> {
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

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let mut output = String::new();

    for file_name in matches {
        let icon = TreeWriter::get_icon_for_name(&file_name, false);
        output.push_str(&format!(
            "\n{} {} {} {} {}\n",
            "───".bright_black(),
            icon,
            file_name.bold().cyan(),
            "───".bright_black(),
            "─".repeat(file_name.len().saturating_sub(3)).bright_black(),
        ));

        let mut content = String::new();
        manager.stream_file(&file_name, |reader| {
            reader
                .read_to_string(&mut content)
                .map_err(|e| ZipCrawlError::IoError {
                    path: file_name.clone(),
                    source: e,
                })?;
            Ok(())
        })?;

        let ext = file_name.split('.').next_back().unwrap_or("");
        let syntax = ss
            .find_syntax_by_extension(ext)
            .or_else(|| match ext {
                "ts" | "tsx" | "mts" | "cts" => ss.find_syntax_by_extension("js"),
                "jsx" | "mjs" | "cjs" | "vue" | "svelte" => ss.find_syntax_by_extension("js"),
                _ => None,
            })
            .unwrap_or_else(|| ss.find_syntax_plain_text());

        let mut highlighter = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

        for (idx, line) in LinesWithEndings::from(&content).enumerate() {
            let line_num = format!("{:>4}", idx + 1).dimmed();
            output.push_str(&format!(" {} │ ", line_num));

            let ranges = highlighter.highlight_line(line, &ss).unwrap_or_default();
            let ansi = as_24_bit_terminal_escaped(&ranges[..], false);
            output.push_str(&ansi);
        }
        output.push('\n');
    }

    let pager = Pager::new();
    pager.set_text(output).ok();
    pager.set_prompt("zipcrawl bat").ok();
    minus::page_all(pager).map_err(|e| ZipCrawlError::IoError {
        path: "pager".into(),
        source: io::Error::other(e),
    })
}
