use crate::archive::ZipManager;
use crate::display::TreeWriter;
use crate::errors::ZipCrawlError;

pub fn handle(manager: &mut ZipManager, show_sizes: bool) -> Result<(), ZipCrawlError> {
    let entries = manager.entries()?;
    let total_size: u64 = entries.iter().map(|e| e.size).sum();

    for entry in entries {
        if !entry.is_dir {
            let icon = TreeWriter::get_icon_for_name(&entry.name, false);
            if show_sizes {
                let size_str = TreeWriter::format_size(entry.size);
                let bar = TreeWriter::get_bar(entry.size, total_size);
                println!("{icon} {0:<40} {size_str:>10} {bar}", entry.name);
            } else {
                println!("{icon} {}", entry.name);
            }
        }
    }
    Ok(())
}
