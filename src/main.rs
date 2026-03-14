use clap::{Parser, Subcommand};
use regex::Regex;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use zip::ZipArchive;

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
        #[arg(short, long, default_value = "2")]
        depth: usize,
    },
    Cat {
        file: String,
    },
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let subcommands = ["tree", "cat", "find", "grep"];

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
            eprintln!("Error in {:?}: {}", path, e);
        }
        println!();
    }

    Ok(())
}

fn process_zip(path: &Path, cmd: &Commands) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    match cmd {
        Commands::Tree { depth } => {
            println!("📦 {:?}", path);
            let mut root = Node::new("root", true);

            for i in 0..archive.len() {
                if let Ok(file) = archive.by_index(i) {
                    let name = file.name();
                    let parts: Vec<&str> = name.split('/').filter(|s| !s.is_empty()).collect();

                    if parts.is_empty() || parts.len() > *depth {
                        continue;
                    }

                    let mut current = &mut root;
                    for (idx, part) in parts.iter().enumerate() {
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
            let mut file = archive.by_name(target_file)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            print!("{}", content);
        }
        Commands::Find { regex } => {
            let re = Regex::new(regex)?;
            for i in 0..archive.len() {
                let file = archive.by_index(i)?;
                if re.is_match(file.name()) {
                    println!("[{:?}] 󰈞 {}", path, file.name());
                }
            }
        }
        Commands::Grep { pattern } => {
            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                if file.is_file() {
                    let mut buffer = Vec::new();
                    if file.read_to_end(&mut buffer).is_ok() {
                        let mut child = Command::new("rg")
                            .arg("--color=always")
                            .arg(pattern)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .spawn()?;

                        if let Some(mut stdin) = child.stdin.take() {
                            let _ = stdin.write_all(&buffer);
                        }

                        let output = child.wait_with_output()?;
                        if !output.stdout.is_empty() {
                            println!("󰈚 Archive: {:?} | File: {}", path, file.name());
                            println!("{}", String::from_utf8_lossy(&output.stdout));
                        }
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
