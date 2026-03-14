use regex::Regex;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use zip::ZipArchive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        println!("Usage: zipcrawl <command> <argument> <files...>");
        println!("\nCommands:");
        println!("  tree <depth>    Show archive structure up to depth");
        println!("  cat  <file>     Print content of a specific file");
        println!("  find <regex>    Search for filenames matching regex");
        println!("  grep <pattern>  Search for text pattern inside all files");
        println!("\nExample: zipcrawl cat config.json ./data.zip");
        std::process::exit(0);
    }

    if args.len() < 4 && args[1] != "tree" {
        eprintln!("Error: Missing arguments. Use -h for help.");
        std::process::exit(1);
    }

    let subcommand = &args[1];
    let target = args.get(2).map(|s| s.as_str()).unwrap_or("2");
    let zip_paths = &args[3..];

    for path_str in zip_paths {
        let path = PathBuf::from(path_str);
        if let Err(e) = process_zip(&path, subcommand, target) {
            eprintln!("Error in {:?}: {}", path, e);
        }
    }

    Ok(())
}

fn process_zip(path: &PathBuf, cmd: &str, target: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    match cmd {
        "tree" => {
            let depth: usize = target.parse().unwrap_or(2);
            println!("📦 Archive: {:?}", path);
            for i in 0..archive.len() {
                let file = archive.by_index(i)?;
                let name = file.name();
                let current_depth = name.split('/').filter(|s| !s.is_empty()).count();
                if current_depth <= depth {
                    let indent = "  ".repeat(current_depth.saturating_sub(1));
                    let icon = if file.is_dir() { "󰉋" } else { "󰈔" };
                    println!("{} {} {}", indent, icon, name);
                }
            }
        }
        "cat" => {
            let mut file = archive.by_name(target)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            println!("{}", content);
        }
        "find" => {
            let re = Regex::new(target)?;
            for i in 0..archive.len() {
                let file = archive.by_index(i)?;
                if re.is_match(file.name()) {
                    println!("[{:?}] 󰈞 {}", path, file.name());
                }
            }
        }
        "grep" => {
            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                if file.is_file() {
                    let mut buffer = Vec::new();
                    if file.read_to_end(&mut buffer).is_ok() {
                        let mut child = Command::new("rg")
                            .arg("--color=always")
                            .arg(target)
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
        _ => eprintln!("Unknown command: {}", cmd),
    }
    Ok(())
}

