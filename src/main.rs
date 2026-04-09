#![deny(clippy::all)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::absolute_paths)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_errors_doc)]

mod archive;
mod commands;
mod display;
mod errors;
mod tui;

use crate::archive::ZipManager;
use crate::commands::{Cli, Commands};
use clap::CommandFactory;
use clap::Parser;
use miette::Result;
use std::env::args;
use std::iter;
use std::path::Path;

fn main() -> Result<()> {
    miette::set_panic_hook();
    let all_args: Vec<String> = args().collect();
    let subcommands = [
        "tree", "cat", "list", "find", "grep", "x", "help", "tui", "diff",
    ];

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
                    Commands::Tree { depth, sizes } => {
                        commands::tree::handle(&mut manager, *depth, *sizes)
                    }
                    Commands::Cat { file, quiet } => {
                        commands::cat::handle(&mut manager, file, *quiet)
                    }
                    Commands::List { sizes } => commands::list::handle(&mut manager, *sizes),
                    Commands::Find {
                        regex,
                        path,
                        glob,
                        entry_type,
                    } => commands::find::handle(
                        &mut manager,
                        regex,
                        path.as_deref(),
                        *glob,
                        entry_type.as_deref(),
                    ),
                    Commands::Grep {
                        pattern,
                        glob,
                        path,
                    } => commands::grep::handle(
                        &mut manager,
                        pattern,
                        glob.as_deref(),
                        path.as_deref(),
                    ),
                    Commands::X {
                        file,
                        command,
                        args,
                        quiet,
                    } => commands::execute::handle(&mut manager, file, command, args, *quiet),
                    Commands::Tui => commands::tui::handle(&mut manager),
                    Commands::Diff {
                        base,
                        exclude,
                        include,
                        mode,
                        quiet,
                    } => commands::diff::handle(
                        &mut manager,
                        base,
                        matches!(mode, commands::DiffMode::Structure),
                        matches!(mode, commands::DiffMode::Stats),
                        matches!(mode, commands::DiffMode::Full),
                        matches!(mode, commands::DiffMode::Full),
                        include.as_deref(),
                        exclude.as_deref(),
                        *quiet,
                    ),
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
            Cli::command().print_help().ok();
        }
    }
    Ok(())
}
