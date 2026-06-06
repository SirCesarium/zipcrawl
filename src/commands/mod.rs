use clap::{Parser, Subcommand};
use clap_complete::shells::Shell;

pub mod bat;
pub mod cat;
pub mod diff;
pub mod execute;
pub mod find;
pub mod grep;
pub mod list;
pub mod tree;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::ValueEnum, Clone, Default)]
pub enum DiffMode {
    #[default]
    Default, // Basic changes
    Structure, // Only names (add/del)
    Stats,     // Names + Sizes
    Full,      // All + Line-by-line diff
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// Display directory structure in a tree format
    #[command(alias = "t")]
    Tree {
        #[arg(short, long, default_value = "4")]
        depth: usize,
        #[arg(short, long)]
        sizes: bool,
    },
    /// Display contents of a file (raw, no headers)
    Cat {
        file: String,

        /// Quiet mode: suppress archive headers, separators and file-not-found errors
        #[arg(short, long)]
        quiet: bool,
    },
    /// Display contents of a file with syntax highlighting
    Bat {
        file: String,
    },
    /// List files and directories
    #[command(aliases = &["ls", "l"])]
    List {
        #[arg(short, long)]
        sizes: bool,
    },
    /// Find files matching a pattern
    #[command(aliases = &["fd", "f"])]
    Find {
        /// Regex pattern to search (or literal string)
        regex: String,

        /// Search only within this subdirectory
        #[arg(short, long)]
        path: Option<String>,

        /// Use glob instead of regex
        #[arg(short, long)]
        glob: bool,

        /// Filter by type: f (file) or d (directory)
        #[arg(short = 't', long)]
        entry_type: Option<String>,
    },
    /// Search for a pattern in files
    #[command(alias = "g")]
    Grep {
        pattern: String,

        /// Only search in files matching this glob (e.g., "*.rs")
        #[arg(short, long)]
        glob: Option<String>,

        /// Filter by subdirectory
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Execute a command on a file
    #[command(aliases = &["exec"])]
    X {
        file: String,
        command: String,
        args: Vec<String>,

        #[arg(short, long)]
        quiet: bool,
    },
    /// Generate shell completion scripts
    #[command(alias = "completion")]
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Compare archives against a base ZIP file
    #[command(alias = "d")]
    Diff {
        /// Base archive for comparison
        #[arg(short, long)]
        base: String,

        /// Comparison detail level
        #[arg(short, long, value_enum, default_value_t = DiffMode::Default)]
        mode: DiffMode,

        /// Patterns to include (positional, e.g. "src/*")
        #[arg(short, long, value_delimiter = ',', value_name = "INCLUDE")]
        include: Option<Vec<String>>,

        /// Patterns to exclude
        #[arg(short, long, value_delimiter = ',')]
        exclude: Option<Vec<String>>,

        /// Quiet mode: output raw diff without pager
        #[arg(short, long)]
        quiet: bool,
    },
}
