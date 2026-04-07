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
use crate::display::{Node, TreeWriter};
use crate::errors::ZipCrawlError;
use clap::{Parser, Subcommand};
use colored::Colorize;
use miette::Result;
use regex::Regex;
use std::env::args;
use std::io::{self, BufRead, BufReader};
use std::iter;
use std::path::Path;
use std::process::{Command, Stdio};

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

fn handle_tree(manager: &mut ZipManager, depth: usize, sizes: bool) -> Result<(), ZipCrawlError> {
    let mut root = Node::new("root", true);
    let entries = manager.entries()?;

    for entry in entries {
        let parts: Vec<&str> = entry.name.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &mut root;
        current.size += entry.size;

        for (idx, part) in parts.iter().enumerate() {
            if idx + 1 > depth {
                break;
            }
            let is_dir = idx < parts.len() - 1 || entry.is_dir;
            current = current
                .children
                .entry((*part).to_string())
                .or_insert_with(|| Node::new(part, is_dir));
            current.size += entry.size;
        }
    }

    let total_size = root.size;
    let count = root.children.len();
    for (i, (_, node)) in root.children.iter().enumerate() {
        TreeWriter::write(node, "", i == count - 1, total_size, sizes);
    }
    Ok(())
}

fn handle_list(manager: &mut ZipManager, show_sizes: bool) -> Result<(), ZipCrawlError> {
    let entries = manager.entries()?;
    let total_size: u64 = entries.iter().map(|e| e.size).sum();

    for entry in entries {
        if !entry.is_dir {
            let icon = TreeWriter::get_icon_for_name(&entry.name, false);
            if show_sizes {
                let size_str = TreeWriter::format_size(entry.size);
                let bar = TreeWriter::get_bar(entry.size, total_size);
                println!("{icon} {0:<40} {size_str:>10} {bar}", entry.name);
            } else {
                println!("{icon} {}", entry.name);
            }
        }
    }
    Ok(())
}

fn handle_grep(manager: &mut ZipManager, pattern: &str) -> Result<(), ZipCrawlError> {
    let re = Regex::new(pattern).map_err(|e| ZipCrawlError::InvalidRegex {
        regex: pattern.to_string(),
        source: e,
    })?;

    let entries = manager.entries()?;
    for entry in entries {
        if !entry.is_dir {
            let mut file = manager.open_file(&entry.name)?;
            let reader = BufReader::new(&mut file);
            let mut matched_lines = Vec::new();

            for line_result in reader.lines() {
                if let Ok(line) = line_result
                    && re.is_match(&line)
                {
                    let highlighted = re
                        .replace_all(&line, |caps: &regex::Captures| {
                            caps[0].red().bold().to_string()
                        })
                        .into_owned();
                    matched_lines.push(highlighted);
                }
            }

            if !matched_lines.is_empty() {
                let icon = TreeWriter::get_icon_for_name(&entry.name, false);
                println!("{} {}", icon, entry.name.cyan().bold());
                for line in matched_lines {
                    println!("  {}", line.trim());
                }
                println!();
            }
        }
    }
    Ok(())
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
                    Commands::Tree { depth, sizes } => handle_tree(&mut manager, *depth, *sizes),
                    Commands::Cat { file } => {
                        let mut entry = manager.open_file(file)?;
                        io::copy(&mut entry, &mut io::stdout())
                            .map(|_| ())
                            .map_err(|e| ZipCrawlError::IoError {
                                path: file.clone(),
                                source: e,
                            })
                    }
                    Commands::List { sizes } => handle_list(&mut manager, *sizes),
                    Commands::Find { regex } => {
                        let re = Regex::new(regex).map_err(|e| ZipCrawlError::InvalidRegex {
                            regex: regex.clone(),
                            source: e,
                        })?;
                        for entry in manager.entries()? {
                            if re.is_match(&entry.name) {
                                println!("[{}] {}", manager.path_name, entry.name);
                            }
                        }
                        Ok(())
                    }
                    Commands::Grep { pattern } => handle_grep(&mut manager, pattern),
                    Commands::X {
                        file,
                        command,
                        args,
                    } => {
                        let mut entry = manager.open_file(file)?;
                        let mut child = Command::new(command)
                            .args(args)
                            .stdin(Stdio::piped())
                            .spawn()
                            .map_err(|e| ZipCrawlError::ExecutionError {
                                cmd: command.clone(),
                                source: e,
                            })?;
                        if let Some(mut stdin) = child.stdin.take() {
                            io::copy(&mut entry, &mut stdin).ok();
                        }
                        child.wait().ok();
                        Ok(())
                    }
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
