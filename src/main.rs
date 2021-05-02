// Copyright 2021 Matthew James Kraai
//
// This file is part of odo.
//
// odo is free software: you can redistribute it and/or modify it under the terms of the GNU Affero
// General Public License as published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// odo is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the
// implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero
// General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with odo.  If not,
// see <https://www.gnu.org/licenses/>.

use directories::ProjectDirs;
use rusqlite::Connection;
#[cfg(all(unix, not(target_os = "macos")))]
use std::os::unix::fs::DirBuilderExt;
use std::{env, fs::DirBuilder, io, process};

fn main() {
    if let Err(e) = run() {
        eprintln!("odo: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let command = parse_args()?;
    let project_dirs = ProjectDirs::from("org.ftbfs", "", "odo")
        .ok_or("unable to determine project directories")?;
    let data_dir = project_dirs.data_dir();
    let mut builder = DirBuilder::new();
    #[cfg(all(unix, not(target_os = "macos")))]
    builder.mode(0o700);
    builder
        .recursive(true)
        .create(data_dir)
        .map_err(|e| format!("unable to create `{}`: {}", data_dir.display(), e))?;
    let database_path = data_dir.join("odo.sqlite3");
    let connection = Connection::open(&database_path)
        .map_err(|e| format!("unable to open `{}`: {}", database_path.display(), e))?;
    odo::initialize(&connection)
        .map_err(|e| format!("unable to initialize `{}`: {}", database_path.display(), e))?;
    match command {
        Command::Action(subcommand) => match subcommand {
            ActionSubcommand::Add { description } => odo::add_action(&connection, description)?,
            ActionSubcommand::List => odo::list_actions(&connection, &mut io::stdout())?,
            ActionSubcommand::Remove { description } => {
                odo::remove_action(&connection, description)?
            }
        },
        Command::Goal(subcommand) => match subcommand {
            GoalSubcommand::Add {
                action,
                description,
            } => odo::add_goal(&connection, description, action)?,
            GoalSubcommand::List => odo::list_goals(&connection, &mut io::stdout())?,
            GoalSubcommand::Remove { description } => {
                if connection
                    .execute(
                        "DELETE FROM goals WHERE description = ?1",
                        rusqlite::params![description],
                    )
                    .map_err(|e| format!("unable to remove goal: {}", e))?
                    != 1
                {
                    return Err("goal does not exist".into());
                }
            }
        },
    }
    Ok(())
}

fn parse_args() -> Result<Command, String> {
    let mut args = env::args().skip(1);
    match args.next() {
        Some(command) => match command.as_str() {
            "action" => match args.next() {
                Some(subcommand) => match subcommand.as_str() {
                    "add" => {
                        let args = args.collect::<Vec<_>>();
                        if args.is_empty() {
                            return Err("missing description".into());
                        }
                        Ok(Command::Action(ActionSubcommand::Add {
                            description: args.join(" "),
                        }))
                    }
                    "ls" => Ok(Command::Action(ActionSubcommand::List)),
                    "rm" => {
                        let args = args.collect::<Vec<_>>();
                        if args.is_empty() {
                            return Err("missing description".into());
                        }
                        Ok(Command::Action(ActionSubcommand::Remove {
                            description: args.join(" "),
                        }))
                    }
                    _ => Err(format!("no such subcommand: `{}`", subcommand)),
                },
                None => Err("missing subcommand".into()),
            },
            "goal" => match args.next() {
                Some(subcommand) => match subcommand.as_str() {
                    "add" => {
                        let mut action = None;
                        let mut args = args.collect::<Vec<_>>();
                        if !args.is_empty() && args[0] == "--action" {
                            args.remove(0);
                            if args.is_empty() {
                                return Err("option `--action` requires an argument".into());
                            }
                            action = Some(args.remove(0));
                        }
                        if args.is_empty() {
                            return Err("missing description".into());
                        }
                        Ok(Command::Goal(GoalSubcommand::Add {
                            action,
                            description: args.join(" "),
                        }))
                    }
                    "ls" => Ok(Command::Goal(GoalSubcommand::List)),
                    "rm" => {
                        let args = args.collect::<Vec<_>>();
                        if args.is_empty() {
                            return Err("missing description".into());
                        }
                        Ok(Command::Goal(GoalSubcommand::Remove {
                            description: args.join(" "),
                        }))
                    }
                    _ => Err(format!("no such subcommand: `{}`", subcommand)),
                },
                None => Err("missing subcommand".into()),
            },
            _ => Err(format!("no such command: `{}`", command)),
        },
        None => Err("missing command".into()),
    }
}

enum Command {
    Action(ActionSubcommand),
    Goal(GoalSubcommand),
}

enum ActionSubcommand {
    Add { description: String },
    List,
    Remove { description: String },
}

enum GoalSubcommand {
    Add {
        action: Option<String>,
        description: String,
    },
    List,
    Remove {
        description: String,
    },
}
