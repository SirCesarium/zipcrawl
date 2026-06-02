#![deny(clippy::all)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::absolute_paths)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_errors_doc)]

mod archive;
mod commands;
mod completions;
mod display;
mod errors;

use crate::archive::ZipManager;
use crate::commands::{Cli, Commands};
use clap::CommandFactory;
use clap::Parser;
use miette::{Report, Result};
use std::env::args;
use std::iter;
use std::path::Path;
use std::process::exit;

fn is_quiet(cmd: &Commands) -> bool {
    match cmd {
        Commands::X { quiet, .. } | Commands::Diff { quiet, .. } => *quiet,
        _ => false,
    }
}

fn main() -> Result<()> {
    miette::set_panic_hook();

    clap_complete::CompleteEnv::with_factory(commands::Cli::command).complete();

    let all_args: Vec<String> = args().collect();
    let subcommands = [
        "tree", "t", "cat", "bat", "list", "ls", "l", "find", "fd", "f", "grep", "g", "x", "exec", "help", "diff", "d", "completions", "completion",
    ];

    let sub_idx = all_args
        .iter()
        .position(|a| subcommands.contains(&a.as_str()));

    match sub_idx {
        Some(idx) if idx >= 1 => {
            let zip_paths = &all_args[1..idx];
            let cmd_args = iter::once(all_args[0].clone()).chain(all_args[idx..].iter().cloned());
            let cli = Cli::parse_from(cmd_args);

            let mut has_errors = false;

            if matches!(&cli.command, Commands::Completions { .. }) {
                let Commands::Completions { shell } = &cli.command else {
                    unreachable!()
                };
                completions::handle(*shell).map_err(|e| errors::ZipCrawlError::IoError {
                    path: "stdout".into(),
                    source: e,
                })?;
                return Ok(());
            }

            for path_str in zip_paths {
                let path = Path::new(path_str);
                let quiet = is_quiet(&cli.command);
                if zip_paths.len() > 1 && !quiet {
                    println!("📦 Archive: {path_str}");
                }

                let manager = ZipManager::new(path);
                let mut manager = match manager {
                    Ok(m) => m,
                    Err(e) => {
                        has_errors = true;
                        eprintln!("{:?}", Report::from(e));
                        continue;
                    }
                };

                let res = match &cli.command {
                    Commands::Tree { depth, sizes } => {
                        commands::tree::handle(&mut manager, *depth, *sizes)
                    }
                    Commands::Cat { file } => {
                        commands::cat::handle(&mut manager, file)
                    }
                    Commands::Bat { file } => {
                        commands::bat::handle(&mut manager, file)
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
                    Commands::Completions { .. } => unreachable!(),
                    Commands::Diff {
                        base,
                        exclude,
                        include,
                        mode,
                        quiet,
                    } => commands::diff::handle(
                        &mut manager,
                        base,
                        !matches!(mode, commands::DiffMode::Default),
                        matches!(mode, commands::DiffMode::Stats) || matches!(mode, commands::DiffMode::Full),
                        matches!(mode, commands::DiffMode::Full),
                        matches!(mode, commands::DiffMode::Full),
                        include.as_deref(),
                        exclude.as_deref(),
                        *quiet,
                    ),
                };

                if let Err(e) = res {
                    has_errors = true;
                    eprintln!("{:?}", Report::from(e));
                }
                if zip_paths.len() > 1 && !quiet {
                    println!("{}", "-".repeat(40));
                }
            }

            if has_errors {
                exit(1);
            }
        }
        _ => {
            Cli::command().print_help().ok();
        }
    }
    Ok(())
}
