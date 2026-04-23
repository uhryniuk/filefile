mod commands;
mod common;
mod filefile;
mod node;
mod operations;

use anyhow::Result;
use common::{get_basename, get_cwd, get_filefile_name, is_directory};
use node::Node;

use commands::Command;
use filefile::FilefileNamesIterator;

fn main() -> Result<()> {
    // Initialize the singleton object
    common::init_global_state();

    let mut command = clap::Command::new("ff")
        .bin_name("ff")
        .subcommand_required(false)
        .arg(
            clap::Arg::new("filefile")
                .action(clap::ArgAction::Set)
                .required(false)
                .index(1)
                .help("Path to a Filefile to apply (shorthand for `ff apply -i <FILE>`)"),
        )
        .arg(
            clap::Arg::new("dry-run")
                .action(clap::ArgAction::SetTrue)
                .required(false)
                .short('d')
                .long("dry-run")
                .help("Simluate operations to execute."),
        )
        .arg(
            clap::Arg::new("force")
                .action(clap::ArgAction::SetTrue)
                .required(false)
                .short('f')
                .long("force")
                .help("Run all operations that throw warning."),
        )
        .arg(
            clap::Arg::new("verbose")
                .action(clap::ArgAction::SetTrue)
                .required(false)
                .short('v')
                .long("verbose")
                .help("Provide more detail action in stderr."),
        )
        .arg(
            clap::Arg::new("allow-remote-ops")
                .action(clap::ArgAction::SetTrue)
                .required(false)
                .long("allow-remote-ops")
                .help("Permit !git and !sh ops in Filefiles fetched over http(s)."),
        )
        .subcommand(
            clap::Command::new("generate")
                .arg(
                    clap::Arg::new("path")
                        .action(clap::ArgAction::Set)
                        .required(false)
                        .short('p')
                        .long("path")
                        .help("Path to contextual root for generating the Filefile"),
                )
                .arg(
                    clap::Arg::new("file")
                        .action(clap::ArgAction::Set)
                        .required(false)
                        .short('o')
                        .long("output")
                        .help("Location and filename of file system to write."),
                )
                .arg(
                    clap::Arg::new("stdout")
                        .action(clap::ArgAction::SetTrue)
                        .required(false)
                        .short('s')
                        .long("stdout")
                        .help("Write config to stdout"),
                ),
        )
        .subcommand(
            clap::Command::new("apply")
                .arg(
                    clap::Arg::new("path")
                        .action(clap::ArgAction::Set)
                        .required(false)
                        .short('p')
                        .long("path")
                        .help("Path to contextual root for generating the Filefile"),
                )
                .arg(
                    clap::Arg::new("file")
                        .action(clap::ArgAction::Set)
                        .required(false)
                        .short('i')
                        .long("input")
                        .help("Location and filename of file system to write."),
                ),
        );
    let matches = command.clone().get_matches();

    // Setting global states.
    // 'ctx' must remain in scope, ref drop
    {
        let ctx = &mut common::get_global_state();
        if matches.get_flag("force") {
            ctx.toggle_force();
        }
        if matches.get_flag("dry-run") {
            ctx.toggle_dry_run();
        }
        if matches.get_flag("verbose") {
            ctx.toggle_verbose();
        }
    }

    let allow_remote_ops = matches.get_flag("allow-remote-ops");

    // Divergence based on subcommands
    match matches.subcommand() {
        Some(("generate", sub_matches)) => {
            let generate = commands::GenerateCommand {
                matches: sub_matches,
            };
            generate.execute()?;
        }
        Some(("apply", sub_matches)) => {
            commands::ApplyCommand::from_subcommand(sub_matches, allow_remote_ops)?.execute()?;
        }
        _ => {
            if let Some(file) = matches.get_one::<String>("filefile") {
                commands::ApplyCommand::from_file(file, allow_remote_ops)?.execute()?;
            } else {
                let _ = command.print_help();
            }
        }
    };

    Ok(())
}
