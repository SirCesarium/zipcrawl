use miette::Diagnostic;
use std::io;
use thiserror::Error;
use zip::result::ZipError;

/// Central error type for the zipcrawl library and CLI.
///
/// Integrates with `miette` for rich terminal error reporting.
#[derive(Error, Debug, Diagnostic)]
pub enum ZipCrawlError {
    /// Error triggered by standard filesystem or pipe operations.
    #[error("Failed to open file at: {path}")]
    #[diagnostic(code(zipcrawl::io_error), help("Verify path and permissions."))]
    IoError {
        path: String,
        #[source]
        source: io::Error,
    },

    /// Errors propagated from the underlying `zip` crate.
    #[error("ZIP archive error")]
    #[diagnostic(code(zipcrawl::zip_error))]
    ZipError(#[from] ZipError),

    /// Triggered when a provided search pattern is not a valid regular expression.
    #[error("Invalid regex pattern: {regex}")]
    #[diagnostic(code(zipcrawl::invalid_regex))]
    InvalidRegex {
        regex: String,
        #[source]
        source: regex::Error,
    },

    /// Errors occurring during the execution of an external command via the `x` command.
    #[error("Command execution failed: {cmd}")]
    #[diagnostic(code(zipcrawl::exec_error))]
    ExecutionError {
        cmd: String,
        #[source]
        source: io::Error,
    },

    /// Requested file was not found within the ZIP internal structure.
    #[error("File '{filename}' not found in archive")]
    #[diagnostic(code(zipcrawl::file_not_found))]
    FileNotFound { filename: String },

    /// Rejection of a file path that attempts to access parent directories via `..`.
    #[error("Invalid path detected (Traversal attempt): {path}")]
    #[diagnostic(
        code(zipcrawl::invalid_path),
        help("Paths cannot contain '..' components.")
    )]
    InvalidPath { path: String },

    /// Safety error when an archive entry appears to be a Zip Bomb.
    #[error("Potential Zip Bomb detected in '{filename}'")]
    #[diagnostic(
        code(zipcrawl::zip_bomb),
        help("The file exceeds size limits or has an abnormal compression ratio.")
    )]
    ZipBombDetected { filename: String },

    /// Invalid glob pattern provided for filtering file entries.
    #[error("Invalid glob pattern: {glob}")]
    #[diagnostic(code(zipcrawl::invalid_glob))]
    InvalidGlob { glob: String },
}
