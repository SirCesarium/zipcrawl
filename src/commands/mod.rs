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
    Tree {
        #[arg(short, long, default_value = "4")]
        depth: usize,
        #[arg(short, long)]
        sizes: bool,
    },
    Cat {
        file: String,
    },
    List {
        #[arg(short, long)]
        sizes: bool,
    },
    Find {
        regex: String,
    },
    Grep {
        pattern: String,
    },
    X {
        file: String,
        command: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    Tui,
}
