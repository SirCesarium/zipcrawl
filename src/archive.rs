use crate::errors::ZipCrawlError;
use core::fmt;
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::path::Component::ParentDir;
use std::path::Path;
use zip::ZipArchive;
use zip::read::ZipFile;

/// Represents a single entry within a ZIP archive.
#[derive(Clone)]
pub struct ZipEntry {
    /// Full path name within the archive.
    pub name: String,
    /// Indicates if the entry is a directory.
    pub is_dir: bool,
    /// Uncompressed size in bytes.
    pub size: u64,

    pub crc: u32,
}

/// Core manager for ZIP archive operations.
///
/// Handles file access, security validations (Zip Bombs/Traversal),
/// and metadata extraction.
pub struct ZipManager {
    archive: ZipArchive<File>,
    /// The source path of the ZIP file on the system.
    pub path_name: String,
}

impl ZipManager {
    /// Ratio at which a file is considered a potential Zip Bomb.
    const MAX_RATIO: u64 = 100;
    /// Maximum allowed uncompressed size (1GB) to prevent memory exhaustion.
    const MAX_SIZE: u64 = 1024 * 1024 * 1024;

    /// Creates a new manager and opens the ZIP archive at the specified path.
    pub fn new(path: &Path) -> Result<Self, ZipCrawlError> {
        let file = File::open(path).map_err(|e| ZipCrawlError::IoError {
            path: path.to_string_lossy().to_string(),
            source: e,
        })?;
        let archive = ZipArchive::new(file)?;
        Ok(Self {
            archive,
            path_name: path.to_string_lossy().to_string(),
        })
    }

    /// Returns a flat list of all entries contained in the archive.
    pub fn entries(&mut self) -> Result<Vec<ZipEntry>, ZipCrawlError> {
        let len = self.archive.len();
        let mut entries = Vec::with_capacity(len);
        for i in 0..len {
            let file = self.archive.by_index(i)?;
            entries.push(ZipEntry {
                name: file.name().to_string(),
                is_dir: file.is_dir(),
                size: file.size(),
                crc: file.crc32(),
            });
        }
        Ok(entries)
    }

    /// Opens a file for reading.
    ///
    /// # Security
    /// - Performs path traversal checks (rejects `..` components).
    /// - Checks for Zip Bomb characteristics (size and compression ratio).
    pub fn open_file(&mut self, name: &str) -> Result<ZipFile<'_, File>, ZipCrawlError> {
        let file = self
            .archive
            .by_name(name)
            .map_err(|_| ZipCrawlError::FileNotFound {
                filename: name.to_string(),
            })?;

        if let Some(enclosed) = file.enclosed_name() {
            if enclosed.components().any(|c| matches!(c, ParentDir)) {
                return Err(ZipCrawlError::InvalidPath {
                    path: name.to_string(),
                });
            }
        } else {
            return Err(ZipCrawlError::InvalidPath {
                path: name.to_string(),
            });
        }

        let compressed = file.compressed_size();
        let uncompressed = file.size();

        if uncompressed > Self::MAX_SIZE {
            return Err(ZipCrawlError::ZipBombDetected {
                filename: name.to_string(),
            });
        }

        if compressed > 0 && (uncompressed / compressed) > Self::MAX_RATIO {
            return Err(ZipCrawlError::ZipBombDetected {
                filename: name.to_string(),
            });
        }

        Ok(file)
    }

    /// Reads the full content of a file into a byte buffer.
    ///
    /// This method is subject to `MAX_SIZE` limits for safety.
    pub fn read_file_content(&mut self, name: &str) -> Result<Vec<u8>, ZipCrawlError> {
        let file = self.open_file(name)?;

        let size = usize::try_from(file.size()).map_err(|_| ZipCrawlError::IoError {
            path: name.to_string(),
            source: Error::new(
                ErrorKind::InvalidData,
                "File size exceeds system architecture limits",
            ),
        })?;

        let mut buffer = Vec::with_capacity(size);

        file.take(Self::MAX_SIZE)
            .read_to_end(&mut buffer)
            .map_err(|e| ZipCrawlError::IoError {
                path: name.to_string(),
                source: e,
            })?;

        Ok(buffer)
    }
}

impl fmt::Debug for ZipManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ZipManager")
            .field("path_name", &self.path_name)
            .field("entries_count", &self.archive.len())
            .finish()
    }
}
