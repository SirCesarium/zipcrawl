mod errors;

use clap::{Parser, Subcommand};
use miette::IntoDiagnostic;
use regex::Regex;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use zip::ZipArchive;

use crate::errors::ZipCrawlError;

#[derive(Parser)]
#[command(name = "zipcrawl")]
struct Cli {
    #[arg(required = true)]
    zip_path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
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
}

struct Node {
    name: String,
    is_dir: bool,
    children: BTreeMap<String, Node>,
}

impl Node {
    fn new(name: &str, is_dir: bool) -> Self {
        Self {
            name: name.to_string(),
            is_dir,
            children: BTreeMap::new(),
        }
    }
}

fn main() -> miette::Result<()> {
    miette::set_panic_hook();

    let args: Vec<String> = std::env::args().collect();
    let subcommands = ["tree", "cat", "list", "find", "grep"];

    let mut zip_paths = Vec::new();
    let mut sub_args = Vec::new();
    let mut found_sub = false;

    for arg in args.into_iter().skip(1) {
        if !found_sub && subcommands.contains(&arg.as_str()) {
            found_sub = true;
        }

        if found_sub {
            sub_args.push(arg);
        } else {
            zip_paths.push(arg);
        }
    }

    if zip_paths.is_empty() || sub_args.is_empty() {
        let _ = Cli::parse_from(vec!["zipcrawl", "--help"]);
        return Ok(());
    }

    let mut dummy_args = vec!["zipcrawl".to_string(), zip_paths[0].clone()];
    dummy_args.extend(sub_args);
    let cli = Cli::parse_from(dummy_args);

    for path_str in zip_paths {
        let path = Path::new(&path_str);
        if let Err(e) = process_zip(path, &cli.command) {
            eprintln!("{:?}", e);
        }
        println!();
    }

    Ok(())
}

fn process_zip(path: &Path, cmd: &Commands) -> miette::Result<()> {
    let file = File::open(path).map_err(|e| ZipCrawlError::IoError {
        path: path.to_string_lossy().to_string(),
        source: e,
    })?;
    let mut archive = ZipArchive::new(file).into_diagnostic()?;

    match cmd {
        Commands::Tree { depth } => {
            println!("📦 {:?}", path);
            let mut root = Node::new("root", true);

            for i in 0..archive.len() {
                if let Ok(file) = archive.by_index(i) {
                    let name = file.name();
                    let parts: Vec<&str> = name.split('/').filter(|s| !s.is_empty()).collect();

                    if parts.is_empty() {
                        continue;
                    }
                    let mut current = &mut root;
                    for (idx, part) in parts.iter().enumerate() {
                        let current_depth = idx + 1;

                        if current_depth > *depth {
                            break;
                        }

                        let is_last = idx == parts.len() - 1;
                        let is_dir = if is_last { name.ends_with('/') } else { true };

                        current = current
                            .children
                            .entry(part.to_string())
                            .or_insert_with(|| Node::new(part, is_dir));
                    }
                }
            }

            let child_count = root.children.len();
            for (i, (_, node)) in root.children.iter().enumerate() {
                draw_node(node, "", i == child_count - 1);
            }
        }
        Commands::Cat { file: target_file } => {
            let mut file =
                archive
                    .by_name(target_file)
                    .map_err(|_| ZipCrawlError::FileNotFound {
                        filename: target_file.clone(),
                    })?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| ZipCrawlError::IoError {
                    path: format!("Inside ZIP: {}", target_file),
                    source: e,
                })?;
            print!("{}", content);
        }
        Commands::List => {
            for i in 0..archive.len() {
                let file = archive.by_index(i).into_diagnostic()?;
                if !file.name().ends_with('/') {
                    println!("{}", file.name());
                }
            }
        }
        Commands::Find { regex } => {
            let re = Regex::new(regex).map_err(|e| ZipCrawlError::InvalidRegex {
                regex: regex.clone(),
                source: e,
            })?;
            for i in 0..archive.len() {
                let file = archive.by_index(i).into_diagnostic()?;
                if re.is_match(file.name()) {
                    println!("[{:?}] 󰈞 {}", path, file.name());
                }
            }
        }

        Commands::Grep { pattern } => {
            for i in 0..archive.len() {
                let mut file = archive.by_index(i).into_diagnostic()?;

                if file.is_file() {
                    let mut child = Command::new("rg")
                        .arg("--color=always")
                        .arg(pattern)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                        .map_err(ZipCrawlError::RipgrepError)?;

                    if let Some(mut stdin) = child.stdin.take()
                        && let Err(e) = std::io::copy(&mut file, &mut stdin)
                    {
                        eprintln!("{:?}", e);
                        continue;
                    }

                    let output = child
                        .wait_with_output()
                        .map_err(ZipCrawlError::RipgrepError)?;

                    if !output.stdout.is_empty() {
                        println!("󰈚 Archive: {:?} | File: {}", path, file.name());
                        println!("{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
            }
        }
    }
    Ok(())
}

fn draw_node(node: &Node, prefix: &str, is_last: bool) {
    let connector = if is_last { "└── " } else { "├── " };
    let icon = if node.is_dir { "󰉋" } else { "󰈔" };

    println!("{}{}{} {}", prefix, connector, icon, node.name);

    let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
    let child_count = node.children.len();

    for (i, (_, child)) in node.children.iter().enumerate() {
        draw_node(child, &new_prefix, i == child_count - 1);
    }
}
