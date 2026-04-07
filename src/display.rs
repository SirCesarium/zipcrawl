use std::collections::BTreeMap;

pub struct TreeWriter;

pub struct Node {
    pub name: String,
    pub is_dir: bool,
    pub children: BTreeMap<String, Self>,
}

impl Node {
    #[must_use]
    pub fn new(name: &str, is_dir: bool) -> Self {
        Self {
            name: name.to_string(),
            is_dir,
            children: BTreeMap::new(),
        }
    }
}

impl TreeWriter {
    pub fn write(node: &Node, prefix: &str, is_last: bool) {
        let connector = if is_last { "└── " } else { "├── " };
        let icon = Self::get_icon(node.is_dir);

        println!("{}{}{} {}", prefix, connector, icon, node.name);

        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        let child_count = node.children.len();

        for (i, (_, child)) in node.children.iter().enumerate() {
            Self::write(child, &new_prefix, i == child_count - 1);
        }
    }

    const fn get_icon(is_dir: bool) -> &'static str {
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
