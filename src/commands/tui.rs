use crate::archive::ZipManager;
use crate::errors::ZipCrawlError;
use crate::tui;

pub fn handle(manager: &mut ZipManager) -> Result<(), ZipCrawlError> {
    tui::handle(manager)
}
