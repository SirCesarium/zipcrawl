#![deny(clippy::all)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::absolute_paths)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_errors_doc)]

pub mod archive;
pub mod display;
pub mod errors;

pub use crate::archive::{ZipEntry, ZipManager};
pub use crate::errors::ZipCrawlError;
