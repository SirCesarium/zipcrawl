use crate::archive::ZipManager;
use crate::display::{Node, TreeWriter};
use crate::errors::ZipCrawlError;

pub fn handle(manager: &mut ZipManager, depth: usize, sizes: bool) -> Result<(), ZipCrawlError> {
    let mut root = Node::new("root", true);
    let entries = manager.entries()?;

    for entry in entries {
        let parts: Vec<&str> = entry.name.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &mut root;
        current.size += entry.size;

        for (idx, part) in parts.iter().enumerate() {
            if idx + 1 > depth {
                break;
            }
            let is_dir = idx < parts.len() - 1 || entry.is_dir;
            current = current
                .children
                .entry((*part).to_string())
                .or_insert_with(|| Node::new(part, is_dir));
            current.size += entry.size;
        }
    }

    let total_size = root.size;
    let count = root.children.len();
    for (i, (_, node)) in root.children.iter().enumerate() {
        TreeWriter::write(node, "", i == count - 1, total_size, sizes);
    }
    Ok(())
}
