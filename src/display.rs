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
        let connector = if is_last { "└── " } else { "├── " };
        let icon = Self::get_icon(node.is_dir);
        let node_size = node.size;

        let mut line = format!("{}{}{} {}", prefix, connector, icon, node.name);

        if show_sizes {
            let size_str = Self::format_size(node_size);
            let bar = Self::get_bar(node_size, total_parent_size);
            line = format!("{line:<40} {size_str:>10} {bar}");
        }

        println!("{line}");

        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        let child_count = node.children.len();

        for (i, (_, child)) in node.children.iter().enumerate() {
            Self::write(
                child,
                &new_prefix,
                i == child_count - 1,
                node_size,
                show_sizes,
            );
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
        let i_usize = i as usize;

        let unit_idx = i_usize.min(units.len() - 1);
        let p = 1024_f64.powi(i32::try_from(unit_idx).unwrap_or(0));
        let s = bytes_f / p;

        format!("{:.1} {}", s, units[unit_idx])
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
        let filled_f = (percentage * 10.0).round();
        let filled = filled_f as usize;

        #[cfg(feature = "nerdfonts")]
        let (full, empty) = ("█", "░");
        #[cfg(not(feature = "nerdfonts"))]
        let (full, empty) = ("#", "-");

        format!(
            "[{}{}] {:>3.0}%",
            full.repeat(filled),
            empty.repeat(10 - filled),
            percentage * 100.0
        )
    }

    #[must_use]
    pub const fn get_icon(is_dir: bool) -> &'static str {
        #[cfg(feature = "nerdfonts")]
        {
            if is_dir { "󰉋" } else { "󰈔" }
        }
        #[cfg(not(feature = "nerdfonts"))]
        {
            if is_dir { "[D]" } else { "[F]" }
        }
    }
}
