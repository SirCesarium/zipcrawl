use crate::display::{Node, TreeWriter};
use crate::errors::ZipCrawlError;
use regex::Regex;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use zip::ZipArchive;
use zip::read::ZipFile;

pub struct ZipManager {
    archive: ZipArchive<File>,
    path_name: String,
}

impl ZipManager {
    pub fn new(path: &Path) -> Result<Self, ZipCrawlError> {
        let file = File::open(path).map_err(|e| ZipCrawlError::IoError {
            path: path.to_string_lossy().to_string(),
            source: e,
        })?;
        let archive = ZipArchive::new(file)?;
        Ok(Self {
            archive,
            path_name: path.to_string_lossy().to_string(),
        })
    }

    fn get_file(&mut self, name: &str) -> Result<ZipFile<'_, File>, ZipCrawlError> {
        self.archive
            .by_name(name)
            .map_err(|_| ZipCrawlError::FileNotFound {
                filename: name.to_string(),
            })
    }

    pub fn tree(&mut self, max_depth: usize, show_sizes: bool) -> Result<(), ZipCrawlError> {
        let mut root = Node::new("root", true);

        for i in 0..self.archive.len() {
            let file = self.archive.by_index(i)?;
            let file_size = file.size();
            let parts: Vec<&str> = file.name().split('/').filter(|s| !s.is_empty()).collect();

            let mut current = &mut root;
            current.size += file_size;

            for (idx, part) in parts.iter().enumerate() {
                if idx + 1 > max_depth {
                    break;
                }

                let is_dir = idx < parts.len() - 1 || file.is_dir();

                current = current
                    .children
                    .entry((*part).to_string())
                    .or_insert_with(|| Node::new(part, is_dir));

                current.size += file_size;
            }
        }

        let total_size = root.size;
        let count = root.children.len();
        for (i, (_, node)) in root.children.iter().enumerate() {
            TreeWriter::write(node, "", i == count - 1, total_size, show_sizes);
        }
        Ok(())
    }

    pub fn cat(&mut self, target: &str) -> Result<(), ZipCrawlError> {
        let mut file = self.get_file(target)?;
        io::copy(&mut file, &mut io::stdout()).map_err(|e| ZipCrawlError::IoError {
            path: target.to_string(),
            source: e,
        })?;
        Ok(())
    }

    pub fn list(&mut self, show_sizes: bool) -> Result<(), ZipCrawlError> {
        let mut total_size: u64 = 0;
        for i in 0..self.archive.len() {
            if let Ok(file) = self.archive.by_index(i) {
                total_size += file.size();
            }
        }

        for i in 0..self.archive.len() {
            let file = self.archive.by_index(i)?;

            if file.is_file() {
                let icon = TreeWriter::get_icon(false);
                let file_name = file.name();

                if show_sizes {
                    let file_size = file.size();
                    let size_str = TreeWriter::format_size(file_size);
                    let bar = TreeWriter::get_bar(file_size, total_size);

                    println!("{icon} {file_name:<40} {size_str:>10} {bar}");
                } else {
                    println!("{icon} {file_name}");
                }
            }
        }
        Ok(())
    }

    pub fn find(&mut self, pattern: &str) -> Result<(), ZipCrawlError> {
        let re = Regex::new(pattern).map_err(|e| ZipCrawlError::InvalidRegex {
            regex: pattern.to_string(),
            source: e,
        })?;
        for i in 0..self.archive.len() {
            let file = self.archive.by_index(i)?;
            if re.is_match(file.name()) {
                println!("[{}] {}", self.path_name, file.name());
            }
        }
        Ok(())
    }

    pub fn grep(&mut self, pattern: &str) -> Result<(), ZipCrawlError> {
        let re = Regex::new(pattern).map_err(|e| ZipCrawlError::InvalidRegex {
            regex: pattern.to_string(),
            source: e,
        })?;
        let mut buffer = String::new();
        for i in 0..self.archive.len() {
            let mut file = self.archive.by_index(i)?;
            if file.is_file() {
                buffer.clear();
                if file.read_to_string(&mut buffer).is_ok() && re.is_match(&buffer) {
                    println!("FILE: {}", file.name());
                    buffer.lines().filter(|l| re.is_match(l)).for_each(|line| {
                        println!("  {}", line.trim());
                    });
                }
            }
        }
        Ok(())
    }

    pub fn execute(
        &mut self,
        target: &str,
        cmd: &str,
        args: &[String],
    ) -> Result<(), ZipCrawlError> {
        let mut file = self.get_file(target)?;
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| ZipCrawlError::ExecutionError {
                cmd: cmd.to_string(),
                source: e,
            })?;

        if let Some(mut stdin) = child.stdin.take() {
            io::copy(&mut file, &mut stdin).ok();
        }
        child.wait().ok();
        Ok(())
    }
}
