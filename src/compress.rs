use crate::errors::ZipCrawlError;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::process::id as process_id;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

const DEFAULT_BUF_SIZE: usize = 128 * 1024;

/// Options for [`ZipCompressor`].
#[derive(Debug, Clone)]
pub struct CompressOptions {
    /// Compression level. Pass `None` for crate default. Range depends on method:
    /// Deflated: 0-9 (default 6), Bzip2: 0-9 (default 6), Zstd: -7-22 (default 3).
    pub level: Option<i64>,
    /// Maximum uncompressed size per file in bytes (default: 4 GiB).
    pub max_file_size: u64,
    /// Overwrite the output file if it exists.
    pub overwrite: bool,
    /// Compression method (default: Deflated).
    pub method: CompressionMethod,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            level: None,
            max_file_size: 4 * 1024 * 1024 * 1024,
            overwrite: false,
            method: CompressionMethod::Deflated,
        }
    }
}

/// A safe and fast ZIP compressor.
///
/// Writes to a temporary file and atomically renames on [`finish`](Self::finish),
/// preventing partial/corrupt archives. Validates all paths against traversal attacks
/// and enforces per-file size limits.
///
/// # Example
/// ```no_run
/// use zipcrawl::compress::ZipCompressor;
/// use std::path::Path;
///
/// let mut compressor = ZipCompressor::new(Path::new("output.zip"))?;
/// compressor.compress_file(Path::new("data.txt"), "data.txt")?;
/// compressor.finish()?;
/// # Ok::<_, zipcrawl::ZipCrawlError>(())
/// ```
pub struct ZipCompressor {
    output_path: PathBuf,
    temp_path: PathBuf,
    writer: Option<ZipWriter<BufWriter<File>>>,
    options: CompressOptions,
    finished: bool,
}

impl ZipCompressor {
    /// Create a new compressor targeting `path` with default options.
    pub fn new(path: &Path) -> Result<Self, ZipCrawlError> {
        Self::with_options(path, CompressOptions::default())
    }

    /// Create a new compressor with custom [`CompressOptions`].
    pub fn with_options(path: &Path, options: CompressOptions) -> Result<Self, ZipCrawlError> {
        if !options.overwrite && path.try_exists().is_ok_and(|e| e) {
            return Err(ZipCrawlError::IoError {
                path: path.to_string_lossy().to_string(),
                source: io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "output file already exists; use CompressOptions::overwrite to override",
                ),
            });
        }

        let (temp_path, file) = create_temp_file(path)?;
        let writer = BufWriter::new(file);
        let zip_writer = ZipWriter::new(writer);

        Ok(Self {
            output_path: path.to_path_buf(),
            temp_path,
            writer: Some(zip_writer),
            options,
            finished: false,
        })
    }

    /// Compress a single file on disk into the archive.
    ///
    /// * `source` – Path to the file on disk.
    /// * `arc_name` – Destination path inside the ZIP archive.
    pub fn compress_file(&mut self, source: &Path, arc_name: &str) -> Result<(), ZipCrawlError> {
        let metadata = fs::metadata(source).map_err(|e| ZipCrawlError::IoError {
            path: source.to_string_lossy().to_string(),
            source: e,
        })?;

        if !metadata.is_file() {
            return Err(ZipCrawlError::IoError {
                path: source.to_string_lossy().to_string(),
                source: io::Error::new(io::ErrorKind::InvalidInput, "source is not a regular file"),
            });
        }

        let size = metadata.len();
        let max_size = self.options.max_file_size;

        if size > max_size {
            return Err(ZipCrawlError::SizeLimitExceeded {
                limit: max_size,
                actual: size,
            });
        }

        self.validate_arc_name(arc_name)?;

        let file = File::open(source).map_err(|e| ZipCrawlError::IoError {
            path: source.to_string_lossy().to_string(),
            source: e,
        })?;

        let options = make_options(&self.options, Some(metadata.permissions().mode()));
        let writer = self.writer()?;
        writer.start_file(arc_name, options)?;

        let mut reader = BufReader::with_capacity(DEFAULT_BUF_SIZE, file);
        let count = io::copy(&mut reader, writer).map_err(|e| ZipCrawlError::IoError {
            path: arc_name.to_string(),
            source: e,
        })?;

        if count != size {
            return Err(ZipCrawlError::IoError {
                path: arc_name.to_string(),
                source: io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!("wrote {count} bytes but expected {size} bytes"),
                ),
            });
        }

        Ok(())
    }

    /// Compress all files from a directory recursively into the archive.
    ///
    /// * `source` – Path to the directory on disk.
    /// * `arc_prefix` – Optional prefix prepended to every path inside the ZIP.
    ///   Pass `""` to place files at the archive root.
    pub fn compress_dir(&mut self, source: &Path, arc_prefix: &str) -> Result<(), ZipCrawlError> {
        if !source.is_dir() {
            return Err(ZipCrawlError::IoError {
                path: source.to_string_lossy().to_string(),
                source: io::Error::new(io::ErrorKind::InvalidInput, "source is not a directory"),
            });
        }

        let canonical_source = source.canonicalize().map_err(|e| ZipCrawlError::IoError {
            path: source.to_string_lossy().to_string(),
            source: e,
        })?;

        let mut entries: Vec<_> = fs::read_dir(source)
            .map_err(|e| ZipCrawlError::IoError {
                path: source.to_string_lossy().to_string(),
                source: e,
            })?
            .filter_map(|e| e.ok())
            .collect();

        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let arc_name = if arc_prefix.is_empty() {
                name
            } else {
                format!("{}/{}", arc_prefix, name)
            };

            let ft = entry.file_type().map_err(|e| ZipCrawlError::IoError {
                path: path.to_string_lossy().to_string(),
                source: e,
            })?;

            if ft.is_dir() {
                if path.canonicalize().ok().as_ref() == Some(&canonical_source) {
                    continue;
                }
                self.add_directory_entry(&arc_name)?;
                self.compress_dir(&path, &arc_name)?;
            } else if ft.is_file() {
                self.compress_file(&path, &arc_name)?;
            }
        }

        Ok(())
    }

    /// Compress raw bytes directly into the archive.
    ///
    /// * `data` – The byte content to store.
    /// * `arc_name` – Destination path inside the ZIP archive.
    pub fn compress_bytes(&mut self, data: &[u8], arc_name: &str) -> Result<(), ZipCrawlError> {
        let size = data.len() as u64;
        let max_size = self.options.max_file_size;

        if size > max_size {
            return Err(ZipCrawlError::SizeLimitExceeded {
                limit: max_size,
                actual: size,
            });
        }

        self.validate_arc_name(arc_name)?;

        let options = make_options(&self.options, None);
        let writer = self.writer()?;
        writer.start_file(arc_name, options)?;
        writer.write_all(data).map_err(|e| ZipCrawlError::IoError {
            path: arc_name.to_string(),
            source: e,
        })?;

        Ok(())
    }

    /// Compress data from a [`Read`]er into the archive.
    ///
    /// * `reader` – Source of bytes (e.g. network stream, in-memory buffer).
    /// * `size` – The exact number of bytes the reader is expected to produce.
    /// * `arc_name` – Destination path inside the ZIP archive.
    pub fn compress_reader<R: Read>(
        &mut self,
        mut reader: R,
        size: u64,
        arc_name: &str,
    ) -> Result<(), ZipCrawlError> {
        let max_size = self.options.max_file_size;

        if size > max_size {
            return Err(ZipCrawlError::SizeLimitExceeded {
                limit: max_size,
                actual: size,
            });
        }

        self.validate_arc_name(arc_name)?;

        let options = make_options(&self.options, None);
        let writer = self.writer()?;
        writer.start_file(arc_name, options)?;

        let mut limited = reader.by_ref().take(max_size);
        let count = io::copy(&mut limited, writer).map_err(|e| ZipCrawlError::IoError {
            path: arc_name.to_string(),
            source: e,
        })?;

        if count != size {
            return Err(ZipCrawlError::IoError {
                path: arc_name.to_string(),
                source: io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!("reader produced {count} bytes but expected {size} bytes"),
                ),
            });
        }

        Ok(())
    }

    /// Finalise the ZIP archive and atomically write it to the output path.
    ///
    /// The file is first written to a temporary file in the same directory,
    /// then atomically renamed to the target path. If this method is not
    /// called, the temporary file is cleaned up on drop.
    pub fn finish(mut self) -> Result<(), ZipCrawlError> {
        let writer = self.writer.take().ok_or_else(|| ZipCrawlError::IoError {
            path: self.output_path.to_string_lossy().to_string(),
            source: io::Error::other("compressor already finished or in an invalid state"),
        })?;

        let mut file = writer.finish().map_err(|e| {
            let _ = fs::remove_file(&self.temp_path);
            ZipCrawlError::ZipError(e)
        })?;

        file.flush().map_err(|e| {
            let _ = fs::remove_file(&self.temp_path);
            ZipCrawlError::IoError {
                path: self.temp_path.to_string_lossy().to_string(),
                source: e,
            }
        })?;

        drop(file);

        fs::rename(&self.temp_path, &self.output_path).map_err(|e| ZipCrawlError::IoError {
            path: self.output_path.to_string_lossy().to_string(),
            source: e,
        })?;

        self.finished = true;
        Ok(())
    }

    // ---- internal helpers ----

    fn writer(&mut self) -> Result<&mut ZipWriter<BufWriter<File>>, ZipCrawlError> {
        self.writer.as_mut().ok_or_else(|| ZipCrawlError::IoError {
            path: self.output_path.to_string_lossy().to_string(),
            source: io::Error::other("compressor has already finished"),
        })
    }

    fn validate_arc_name(&self, name: &str) -> Result<(), ZipCrawlError> {
        if name.is_empty() {
            return Err(ZipCrawlError::InvalidPath {
                path: name.to_string(),
            });
        }

        if name.contains('\0') {
            return Err(ZipCrawlError::InvalidPath {
                path: name.to_string(),
            });
        }

        let path = Path::new(name);
        if path.is_absolute() {
            return Err(ZipCrawlError::InvalidPath {
                path: name.to_string(),
            });
        }

        if path.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err(ZipCrawlError::InvalidPath {
                path: name.to_string(),
            });
        }

        Ok(())
    }

    fn add_directory_entry(&mut self, arc_name: &str) -> Result<(), ZipCrawlError> {
        self.validate_arc_name(arc_name)?;
        let options = make_options(&self.options, None);
        let writer = self.writer()?;
        writer.add_directory(arc_name, options)?;
        Ok(())
    }
}

impl Drop for ZipCompressor {
    fn drop(&mut self) {
        if !self.finished {
            self.writer = None;
            let _ = fs::remove_file(&self.temp_path);
        }
    }
}

// ---- helper utilities ----

fn make_options(options: &CompressOptions, mode: Option<u32>) -> SimpleFileOptions {
    let mut opts = SimpleFileOptions::default().compression_method(options.method);

    if let Some(level) = options.level {
        opts = opts.compression_level(Some(level));
    }

    if let Some(mode) = mode {
        opts = opts.unix_permissions(mode);
    }

    opts
}

fn create_temp_file(path: &Path) -> Result<(PathBuf, File), ZipCrawlError> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let file_name = path
        .file_name()
        .map(|n| format!(".{}.{}.tmp", n.to_string_lossy(), process_id()))
        .unwrap_or_else(|| format!(".zipcrawl.{}.tmp", process_id()));

    let temp_path = parent.join(&file_name);

    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
    {
        Ok(file) => Ok((temp_path, file)),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            // Stale temp file from a previous crash; remove and retry.
            let _ = fs::remove_file(&temp_path);
            let file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_path)
                .map_err(|e| ZipCrawlError::IoError {
                    path: temp_path.to_string_lossy().to_string(),
                    source: e,
                })?;
            Ok((temp_path, file))
        }
        Err(e) => Err(ZipCrawlError::IoError {
            path: temp_path.to_string_lossy().to_string(),
            source: e,
        }),
    }
}
