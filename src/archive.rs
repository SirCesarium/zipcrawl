use crate::errors::ZipCrawlError;
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::path::Component::ParentDir;
use std::path::Path;
use zip::ZipArchive;
use zip::read::ZipFile;

pub struct ZipEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

pub struct ZipManager {
    archive: ZipArchive<File>,
    pub path_name: String,
}

impl ZipManager {
    const MAX_RATIO: u64 = 100;
    const MAX_SIZE: u64 = 1024 * 1024 * 1024;

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

    pub fn entries(&mut self) -> Result<Vec<ZipEntry>, ZipCrawlError> {
        let len = self.archive.len();
        let mut entries = Vec::with_capacity(len);
        for i in 0..len {
            let file = self.archive.by_index(i)?;
            entries.push(ZipEntry {
                name: file.name().to_string(),
                is_dir: file.is_dir(),
                size: file.size(),
            });
        }
        Ok(entries)
    }

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

    #[allow(dead_code)]
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
