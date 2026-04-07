use miette::Diagnostic;
use std::io;
use thiserror::Error;
use zip::result::ZipError;

#[derive(Error, Debug, Diagnostic)]
pub enum ZipCrawlError {
    #[error("Failed to open file at: {path}")]
    #[diagnostic(code(zipcrawl::io_error), help("Verify path and permissions."))]
    IoError {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("ZIP archive error")]
    #[diagnostic(code(zipcrawl::zip_error))]
    ZipError(#[from] ZipError),

    #[error("Invalid regex pattern: {regex}")]
    #[diagnostic(code(zipcrawl::invalid_regex))]
    InvalidRegex {
        regex: String,
        #[source]
        source: regex::Error,
    },

    #[error("Command execution failed: {cmd}")]
    #[diagnostic(code(zipcrawl::exec_error))]
    ExecutionError {
        cmd: String,
        #[source]
        source: io::Error,
    },

    #[error("File '{filename}' not found in archive")]
    #[diagnostic(code(zipcrawl::file_not_found))]
    FileNotFound { filename: String },
}
