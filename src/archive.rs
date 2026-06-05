use crate::errors::ZipCrawlError;
use core::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Component::ParentDir;
use std::path::Path;
use zip::read::ZipFile;
use zip::ZipArchive;

#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;

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
    /// Keeps a temp file alive when the archive was created from a reader.
    #[allow(dead_code)]
    _tempfile: Option<tempfile::NamedTempFile>,
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
            _tempfile: None,
        })
    }

    /// Creates a manager from any `Read` source (e.g. stdin, network).
    ///
    /// The content is buffered into a temporary file on disk, then opened
    /// as a standard ZIP archive. The temp file is kept alive for the
    /// lifetime of `ZipManager`.
    #[allow(dead_code)]
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, ZipCrawlError> {
        let mut tmp = tempfile::NamedTempFile::new().map_err(|e| ZipCrawlError::IoError {
            path: String::from("<tempfile>"),
            source: e,
        })?;
        io::copy(reader, &mut tmp).map_err(|e| ZipCrawlError::IoError {
            path: String::from("<stream>"),
            source: e,
        })?;
        let path = tmp.path().to_owned();
        let file = File::open(&path).map_err(|e| ZipCrawlError::IoError {
            path: path.to_string_lossy().to_string(),
            source: e,
        })?;
        let archive = ZipArchive::new(file)?;
        Ok(Self {
            archive,
            path_name: String::from("<stream>"),
            _tempfile: Some(tmp),
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

    /// Opens a file entry for reading, returning a handle that implements [`Read`].
    ///
    /// # Security
    /// - Performs path traversal checks (rejects `..` components).
    /// - Validates against Zip Bomb characteristics (abnormal compression ratios or excessive size).
    pub fn open_file(&mut self, name: &str) -> Result<ZipFile<'_, File>, ZipCrawlError> {
        let file = self
            .archive
            .by_name(name)
            .map_err(|_| ZipCrawlError::FileNotFound {
                filename: name.to_string(),
            })?;

        // Security Check: Path Traversal
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

        // Security Check: Zip Bomb detection
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

    /// Provides streaming access to a file's content via a closure.
    ///
    /// This is the preferred way to process large files as it avoids loading
    /// the entire content into memory (Heap).
    ///
    /// # Example
    /// ```no_run
    /// use std::io;
    /// # use zipcrawl::archive::ZipManager;
    /// # use std::path::Path;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut manager = ZipManager::new(Path::new("example.zip"))?;
    /// manager.stream_file("data.txt", |reader| {
    ///     io::copy(reader, &mut io::stdout()).expect("Failed to copy content to stdout");
    ///     Ok(())
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn stream_file<F, T>(&mut self, name: &str, mut f: F) -> Result<T, ZipCrawlError>
    where
        F: FnMut(&mut ZipFile<'_, File>) -> Result<T, ZipCrawlError>,
    {
        let mut file = self.open_file(name)?;
        f(&mut file)
    }

    /// Reads the full contents of a named entry into a [`String`].
    ///
    /// Internally calls [`open_file`](Self::open_file), so all security checks
    /// (path traversal, Zip Bomb) apply.
    #[allow(dead_code)]
    pub fn read_to_string(&mut self, name: &str) -> Result<String, ZipCrawlError> {
        let path = self.path_name.clone();
        let mut file = self.open_file(name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| ZipCrawlError::IoError { path, source: e })?;
        Ok(content)
    }

    /// Reads and deserializes a named entry using serde.
    ///
    /// The format is chosen based on the file extension:
    /// - `.json` → `serde_json`
    /// - `.toml` → `toml`
    ///
    /// Requires the `serde` feature.
    #[cfg(feature = "serde")]
    #[allow(dead_code)]
    pub fn read_and_deserialize<T: DeserializeOwned>(
        &mut self,
        name: &str,
    ) -> Result<T, ZipCrawlError> {
        let raw = self.read_to_string(name)?;
        let path = Path::new(name);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_lowercase);

        match ext.as_deref() {
            Some("json") => serde_json::from_str(&raw).map_err(|e| ZipCrawlError::ParseError {
                file: name.to_string(),
                details: e.to_string(),
            }),
            Some("toml") => toml::from_str(&raw).map_err(|e| ZipCrawlError::ParseError {
                file: name.to_string(),
                details: e.to_string(),
            }),
            _ => Err(ZipCrawlError::ParseError {
                file: name.to_string(),
                details: format!("unsupported extension: {:?}", ext),
            }),
        }
    }

    /// Extracts all entries matching `prefix` to `destination` on disk.
    ///
    /// Preserves directory structure relative to `prefix`. Security validations
    /// (path traversal) are applied to every extracted entry.
    #[allow(dead_code)]
    pub fn extract_prefix(
        &mut self,
        prefix: &str,
        destination: &Path,
    ) -> Result<(), ZipCrawlError> {
        let matches: Vec<(String, bool)> = self
            .entries()?
            .into_iter()
            .filter(|e| e.name.starts_with(prefix))
            .map(|e| (e.name, e.is_dir))
            .collect();

        for (name, is_dir) in matches {
            let Some(relative) = name.strip_prefix(prefix) else {
                continue;
            };
            let relative_path = Path::new(relative);

            if relative_path.components().any(|c| matches!(c, ParentDir)) {
                return Err(ZipCrawlError::InvalidPath { path: name });
            }

            let out_path = destination.join(relative);

            if !out_path.starts_with(destination) {
                return Err(ZipCrawlError::InvalidPath { path: name });
            }

            if is_dir {
                fs::create_dir_all(&out_path).map_err(|e| ZipCrawlError::ExtractError {
                    entry: name.clone(),
                    destination: out_path.to_string_lossy().to_string(),
                    source: e,
                })?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| ZipCrawlError::ExtractError {
                        entry: name.clone(),
                        destination: parent.to_string_lossy().to_string(),
                        source: e,
                    })?;
                }
                let mut file = self.open_file(&name)?;
                let mut out_file =
                    File::create(&out_path).map_err(|e| ZipCrawlError::ExtractError {
                        entry: name.clone(),
                        destination: out_path.to_string_lossy().to_string(),
                        source: e,
                    })?;
                io::copy(&mut file, &mut out_file).map_err(|e| ZipCrawlError::ExtractError {
                    entry: name.clone(),
                    destination: out_path.to_string_lossy().to_string(),
                    source: e,
                })?;
            }
        }

        Ok(())
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::absolute_paths)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use zip::ZipWriter;

    fn create_test_zip(contents: &[(&str, &str)]) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.zip");
        let file = File::create(&path).unwrap();
        let mut zip = ZipWriter::new(file);
        for (name, content) in contents {
            zip.start_file::<&str, ()>(name, Default::default())
                .unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
        (dir, path)
    }

    fn make_zip_bytes(contents: &[(&str, &str)]) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(&mut buf);
        for (name, content) in contents {
            zip.start_file::<&str, ()>(name, Default::default())
                .unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
        buf.into_inner()
    }

    #[test]
    fn read_to_string_returns_content() {
        let (_d, path) = create_test_zip(&[("hello.txt", "Hello, World!")]);
        let mut mgr = ZipManager::new(&path).unwrap();
        let content = mgr.read_to_string("hello.txt").unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn read_to_string_file_not_found() {
        let (_d, path) = create_test_zip(&[("exists.txt", "content")]);
        let mut mgr = ZipManager::new(&path).unwrap();
        let err = mgr.read_to_string("missing.txt").unwrap_err();
        assert!(matches!(err, ZipCrawlError::FileNotFound { .. }));
    }

    #[test]
    fn read_to_string_file_not_found_on_traversal() {
        let (_d, path) = create_test_zip(&[("safe.txt", "content")]);
        let mut mgr = ZipManager::new(&path).unwrap();
        let err = mgr.read_to_string("../etc/passwd").unwrap_err();
        // The zip crate normalizes `..`, so this becomes a file-not-found error
        assert!(matches!(err, ZipCrawlError::FileNotFound { .. }));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn read_and_deserialize_json() {
        let (_d, path) = create_test_zip(&[("data.json", r#"{"name":"test","value":42}"#)]);
        let mut mgr = ZipManager::new(&path).unwrap();
        #[derive(serde::Deserialize, PartialEq, Debug)]
        struct Data {
            name: String,
            value: i32,
        }
        let data: Data = mgr.read_and_deserialize("data.json").unwrap();
        assert_eq!(
            data,
            Data {
                name: "test".into(),
                value: 42
            }
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn read_and_deserialize_toml() {
        let (_d, path) = create_test_zip(&[(
            "config.toml",
            r#"name = "test"
value = 42"#,
        )]);
        let mut mgr = ZipManager::new(&path).unwrap();
        #[derive(serde::Deserialize, PartialEq, Debug)]
        struct Data {
            name: String,
            value: i32,
        }
        let data: Data = mgr.read_and_deserialize("config.toml").unwrap();
        assert_eq!(
            data,
            Data {
                name: "test".into(),
                value: 42
            }
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn read_and_deserialize_unsupported_ext() {
        let (_d, path) = create_test_zip(&[("data.yaml", "foo: bar")]);
        let mut mgr = ZipManager::new(&path).unwrap();
        let err = mgr
            .read_and_deserialize::<serde_json::Value>("data.yaml")
            .unwrap_err();
        assert!(matches!(err, ZipCrawlError::ParseError { .. }));
    }

    #[test]
    fn extract_prefix_extracts_matching_entries() {
        let (_d, path) = create_test_zip(&[
            ("overrides/config.yml", "config"),
            ("overrides/scripts/init.lua", "init"),
            ("other/file.txt", "other"),
        ]);
        let dest = tempfile::tempdir().unwrap();
        let mut mgr = ZipManager::new(&path).unwrap();
        mgr.extract_prefix("overrides/", dest.path()).unwrap();

        let config_path = dest.path().join("config.yml");
        let script_path = dest.path().join("scripts/init.lua");
        let other_path = dest.path().join("other/file.txt");

        assert!(config_path.exists(), "config.yml should exist");
        assert!(script_path.exists(), "scripts/init.lua should exist");
        assert!(!other_path.exists(), "other/file.txt should NOT exist");
        assert_eq!(fs::read_to_string(&config_path).unwrap(), "config");
        assert_eq!(fs::read_to_string(&script_path).unwrap(), "init");
    }

    #[test]
    fn extract_prefix_empty_prefix_extracts_all() {
        let (_d, path) = create_test_zip(&[("a.txt", "a"), ("b.txt", "b")]);
        let dest = tempfile::tempdir().unwrap();
        let mut mgr = ZipManager::new(&path).unwrap();
        mgr.extract_prefix("", dest.path()).unwrap();
        assert!(dest.path().join("a.txt").exists());
        assert!(dest.path().join("b.txt").exists());
    }

    #[test]
    fn extract_prefix_no_match_does_nothing() {
        let (_d, path) = create_test_zip(&[("file.txt", "content")]);
        let dest = tempfile::tempdir().unwrap();
        let mut mgr = ZipManager::new(&path).unwrap();
        mgr.extract_prefix("nonexistent/", dest.path()).unwrap();
        assert!(dest.path().read_dir().unwrap().next().is_none());
    }

    #[test]
    fn extract_prefix_relative_rejects_parent_dir_components() {
        let (_d, path) = create_test_zip(&[("overrides/../outside.txt", "evil")]);
        let dest = tempfile::tempdir().unwrap();
        let mut mgr = ZipManager::new(&path).unwrap();
        let err = mgr.extract_prefix("", dest.path()).unwrap_err();
        assert!(matches!(err, ZipCrawlError::InvalidPath { .. }));
        assert!(!dest.path().join("outside.txt").exists());
    }

    #[test]
    fn from_reader_reads_entry() {
        let bytes = make_zip_bytes(&[("hello.txt", "Hello from reader!")]);
        let mut reader = std::io::Cursor::new(bytes);
        let mut mgr = ZipManager::from_reader(&mut reader).unwrap();
        assert_eq!(mgr.path_name, "<stream>");
        let entries = mgr.entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "hello.txt");
        let content = mgr.read_to_string("hello.txt").unwrap();
        assert_eq!(content, "Hello from reader!");
    }

    #[test]
    fn from_reader_multiple_entries() {
        let bytes = make_zip_bytes(&[("a.txt", "aaa"), ("b.txt", "bbb"), ("c.txt", "ccc")]);
        let mut reader = std::io::Cursor::new(bytes);
        let mut mgr = ZipManager::from_reader(&mut reader).unwrap();
        let entries = mgr.entries().unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(mgr.read_to_string("b.txt").unwrap(), "bbb");
    }

    #[test]
    fn from_reader_invalid_zip_errors() {
        let mut reader = std::io::Cursor::new(b"not a zip file");
        let err = ZipManager::from_reader(&mut reader).unwrap_err();
        assert!(matches!(err, ZipCrawlError::ZipError(_)));
    }
}
