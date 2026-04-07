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
    },
    Cat {
        file: String,
    },
    List,
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

    let args: Vec<String> = args().collect();
    let subcommands = ["tree", "cat", "list", "find", "grep", "x", "help"];

    let sub_idx = args.iter().position(|a| subcommands.contains(&a.as_str()));

    match sub_idx {
        Some(idx) if idx > 1 => {
            let zip_paths = &args[1..idx];
            let sub_args = &args[idx..];

            let mut cmd_args = vec!["zipcrawl".to_string()];
            cmd_args.extend(sub_args.iter().cloned());
            let cli = Cli::parse_from(cmd_args);

            for path_str in zip_paths {
                let path = Path::new(path_str);

                if zip_paths.len() > 1 {
                    println!("📦 Archive: {path_str}");
                }

                let mut manager = ZipManager::new(path)?;

                let res = match &cli.command {
                    Commands::Tree { depth } => manager.tree(*depth),
                    Commands::Cat { file } => manager.cat(file),
                    Commands::List => manager.list(),
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
