use colored::Colorize;
use similar::ChangeTag;
use std::collections::BTreeMap;

pub struct TreeWriter;

pub struct Node {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub children: BTreeMap<String, Self>,
}

impl Node {
    #[must_use]
    pub fn new(name: &str, is_dir: bool) -> Self {
        Self {
            name: name.to_string(),
            is_dir,
            size: 0,
            children: BTreeMap::new(),
        }
    }
}

impl TreeWriter {
    pub fn write(
        node: &Node,
        prefix: &str,
        is_last: bool,
        total_parent_size: u64,
        show_sizes: bool,
    ) {
        let connector = if is_last { "└── " } else { "├── " }.bright_black();
        let icon = Self::get_icon_for_name(&node.name, node.is_dir);

        let name_display = if node.is_dir {
            node.name.bold().yellow()
        } else {
            node.name.normal()
        };

        let mut line = format!(
            "{}{}{} {}",
            prefix.bright_black(),
            connector,
            icon,
            name_display
        );

        if show_sizes {
            let size_str = Self::format_size(node.size).green();
            let bar = Self::get_bar(node.size, total_parent_size);
            line = format!("{line:<50} {size_str:>10} {bar}");
        }

        println!("{line}");

        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        let child_count = node.children.len();

        for (i, (_, child)) in node.children.iter().enumerate() {
            Self::write(
                child,
                &new_prefix,
                i == child_count - 1,
                node.size,
                show_sizes,
            );
        }
    }

    #[must_use]
    pub fn get_icon_for_name(name: &str, is_dir: bool) -> String {
        #[cfg(not(feature = "nerdfonts"))]
        {
            return (if is_dir { "[D]" } else { "[F]" })
                .bright_blue()
                .to_string();
        }

        #[cfg(feature = "nerdfonts")]
        {
            if is_dir {
                return "󰉋".yellow().to_string();
            }

            let extension = name.split('.').next_back().unwrap_or("").to_lowercase();

            let (icon, color) = match extension.as_str() {
                "rs" => ("", "bright_red"),
                "js" | "mjs" => ("", "yellow"),
                "ts" => ("", "blue"),
                "py" => ("", "blue"),
                "go" => ("", "cyan"),
                "rb" => ("", "red"),
                "php" => ("", "magenta"),
                "java" | "jar" => ("", "red"),
                "cpp" | "cc" => ("", "blue"),
                "c" => ("", "blue"),
                "html" | "htm" => ("", "bright_red"),
                "css" | "scss" => ("", "blue"),
                "json" => ("", "yellow"),
                "md" => ("", "bright_black"),
                "toml" | "yaml" | "yml" => ("", "blue"),
                "xml" => ("󰗀", "bright_red"),
                "zip" | "tar" | "gz" | "7z" => ("󰿺", "yellow"),
                "pdf" => ("", "red"),
                "jpg" | "png" | "svg" | "gif" => ("󰈔", "magenta"),
                "sh" | "bash" | "zsh" => ("", "green"),
                "exe" | "bin" => ("", "bright_red"),
                "txt" => ("󰈙", "bright_black"),
                "sql" => ("", "bright_black"),
                "lock" => ("", "yellow"),
                _ => ("󰈔", "bright_blue"),
            };

            icon.color(color).to_string()
        }
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    pub fn format_size(bytes: u64) -> String {
        let units = ["B", "KB", "MB", "GB", "TB"];
        if bytes == 0 {
            return "0 B".to_string();
        }
        let bytes_f = bytes as f64;
        let i = bytes_f.log(1024.0).floor();
        let unit_idx = (i as usize).min(units.len() - 1);
        let p = 1024_f64.powi(i32::try_from(unit_idx).unwrap_or(0));
        format!("{:.1} {}", bytes_f / p, units[unit_idx])
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    pub fn get_bar(size: u64, total: u64) -> String {
        if total == 0 {
            return String::new();
        }
        let percentage = (size as f64 / total as f64).clamp(0.0, 1.0);
        let filled = (percentage * 10.0).round() as usize;

        #[cfg(feature = "nerdfonts")]
        let (full, empty) = ("█", "░");
        #[cfg(not(feature = "nerdfonts"))]
        let (full, empty) = ("#", "-");

        let bar = format!(
            "{}{}",
            full.repeat(filled).green(),
            empty.repeat(10 - filled).bright_black()
        );

        let percentage_display = format!("{:>3.0}%", percentage * 100.0).yellow();

        format!("[{bar}] {percentage_display}")
    }

    pub fn print_file_header(filename: &str) {
        let icon = Self::get_icon_for_name(filename, false);
        let separator = "─".repeat(filename.len() + 4).bright_black();

        println!("{separator}");
        println!("{} {}", icon, filename.bold().cyan());
        println!("{separator}");
    }
}

pub struct DiffWriter;

impl DiffWriter {
    pub fn format_addition(path: &str, is_dir: bool) -> String {
        let icon = TreeWriter::get_icon_for_name(path, is_dir);
        format!("{} {} {path}", "+".green().bold(), icon)
    }

    pub fn format_removal(path: &str, is_dir: bool) -> String {
        let icon = TreeWriter::get_icon_for_name(path, is_dir);
        format!("{} {} {path}", "-".red().bold(), icon)
    }

    pub fn format_change(path: &str, is_dir: bool, diffs: &[String]) -> String {
        let icon = TreeWriter::get_icon_for_name(path, is_dir);
        let tags = diffs.join(", ");
        format!(
            "{} {} {path} {}",
            "Δ".blue().bold(),
            icon,
            format!("({tags})").italic().dimmed()
        )
    }

    pub fn format_line_diff(change_tag: ChangeTag, content: &str) -> String {
        match change_tag {
            ChangeTag::Delete => format!("{}{}", "-".red(), content.red()),
            ChangeTag::Insert => format!("{}{}", "+".green(), content.green()),
            ChangeTag::Equal => format!(" {content}"),
        }
    }

    /// Cabecera para el bloque de cambios de línea
    pub fn line_diff_header() -> String {
        "--- Line Changes ---".dimmed().to_string()
    }

    /// Separador para el bloque de cambios de línea
    pub fn line_diff_footer() -> String {
        "--------------------".dimmed().to_string()
    }
}
