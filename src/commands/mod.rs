use clap::{Parser, Subcommand};

pub mod cat;
pub mod execute;
pub mod find;
pub mod grep;
pub mod list;
pub mod tree;
pub mod tui;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// Display directory structure in a tree format
    Tree {
        #[arg(short, long, default_value = "4")]
        depth: usize,
        #[arg(short, long)]
        sizes: bool,
    },
    /// Display contents of a file
    Cat { file: String },
    /// List files and directories
    List {
        #[arg(short, long)]
        sizes: bool,
    },
    /// Find files matching a pattern
    Find { regex: String },
    /// Search for a pattern in files
    Grep { pattern: String },
    /// Execute a command on a file
    X {
        file: String,
        command: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Interactive TUI file explorer
    Tui,
}
