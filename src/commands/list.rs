use crate::archive::ZipManager;
use crate::display::TreeWriter;
use crate::errors::ZipCrawlError;

pub fn handle(manager: &mut ZipManager, show_sizes: bool) -> Result<(), ZipCrawlError> {
    let mut entries = manager.entries()?;
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    let total_size: u64 = entries.iter().map(|e| e.size).sum();

    for entry in entries {
        let icon = TreeWriter::get_icon_for_name(&entry.name, entry.is_dir);
        if show_sizes {
            let size_str = TreeWriter::format_size(entry.size);
            let bar = TreeWriter::get_bar(entry.size, total_size);
            println!("{icon} {0:<40} {size_str:>10} {bar}", entry.name);
        } else {
            println!("{icon} {}", entry.name);
        }
    }
    Ok(())
}
