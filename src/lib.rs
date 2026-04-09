//! # `ZipCrawl`
//!
//! A high-performance library and CLI tool to explore, search, and stream
//! contents from ZIP archives without full extraction.
//!
//! This crate provides tools for tree visualization, content searching (grep),
//! and secure file access with protection against Zip Bombs and path traversal.
#![deny(clippy::all)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::absolute_paths)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_errors_doc)]

pub mod archive;
pub mod errors;

pub use crate::archive::{ZipEntry, ZipManager};
pub use crate::errors::ZipCrawlError;
