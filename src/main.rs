#![deny(clippy::all)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::absolute_paths)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_errors_doc)]

mod archive;
mod display;
mod errors;

use crate::archive::ZipManager;
use clap::{Parser, Subcommand};
use miette::Result;
use std::env::args;
use std::iter;
use std::path::Path;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
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
}

fn main() -> Result<()> {
    miette::set_panic_hook();

    let all_args: Vec<String> = args().collect();
    let subcommands = ["tree", "cat", "list", "find", "grep", "x", "help"];

    let sub_idx = all_args
        .iter()
        .position(|a| subcommands.contains(&a.as_str()));

    match sub_idx {
        Some(idx) if idx >= 1 => {
            let zip_paths = &all_args[1..idx];

            let cmd_args = iter::once(all_args[0].clone()).chain(all_args[idx..].iter().cloned());

            let cli = Cli::parse_from(cmd_args);

            for path_str in zip_paths {
                let path = Path::new(path_str);

                if zip_paths.len() > 1 {
                    println!("📦 Archive: {path_str}");
                }

                let mut manager = ZipManager::new(path)?;

                let res = match &cli.command {
                    Commands::Tree { depth, sizes } => manager.tree(*depth, *sizes),
                    Commands::Cat { file } => manager.cat(file),
                    Commands::List { sizes } => manager.list(*sizes),
                    Commands::Find { regex } => manager.find(regex),
                    Commands::Grep { pattern } => manager.grep(pattern),
                    Commands::X {
                        file,
                        command,
                        args,
                    } => manager.execute(file, command, args),
                };

                if let Err(e) = res {
                    eprintln!("Error processing {path_str}: {e:?}");
                }

                if zip_paths.len() > 1 {
                    println!("{}", "-".repeat(40));
                }
            }
        }
        _ => {
            let _ = Cli::parse_from(vec!["zipcrawl", "--help"]);
        }
    }

    Ok(())
}
