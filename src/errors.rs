use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum ZipCrawlError {
    #[error("Failed to open ZIP archive: {path}")]
    #[diagnostic(
        code(zipcrawl::io_error),
        help("Check if the file exists and you have the necessary read permissions.")
    )]
    IoError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse ZIP structure")]
    #[diagnostic(
        code(zipcrawl::zip_format),
        help("The file might be corrupted or is not a valid ZIP archive.")
    )]
    ZipError(#[from] zip::result::ZipError),

    #[error("Invalid regular expression: {regex}")]
    #[diagnostic(
        code(zipcrawl::invalid_regex),
        help("Verify your regex syntax. Example: '.*\\.json$'")
    )]
    InvalidRegex {
        regex: String,
        #[source]
        source: regex::Error,
    },

    #[error("Ripgrep (rg) execution failed")]
    #[diagnostic(
        code(zipcrawl::ripgrep_error),
        help("Make sure 'ripgrep' is installed and available in your PATH.")
    )]
    RipgrepError(#[source] std::io::Error),

    #[error("File not found inside archive: {filename}")]
    #[diagnostic(
        code(zipcrawl::file_not_found),
        help("Use 'zipcrawl list' to see all available files in the archive.")
    )]
    FileNotFound { filename: String },
}
