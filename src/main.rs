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
use std::{env, fs::DirBuilder, process};

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
    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS actions (description PRIMARY KEY)",
            [],
        )
        .map_err(|e| format!("unable to create actions table: {}", e))?;
    match command {
        Command::Action(subcommand) => match subcommand {
            ActionSubcommand::Add { description } => {
                connection
                    .execute(
                        "INSERT INTO actions VALUES(?1)",
                        rusqlite::params![description],
                    )
                    .map_err(|e| format!("unable to add action: {}", e))?;
            }
            ActionSubcommand::Remove { description } => {
                connection
                    .execute(
                        "DELETE FROM actions WHERE description = ?1",
                        rusqlite::params![description],
                    )
                    .map_err(|e| format!("unable to remove action: {}", e))?;
            }
            ActionSubcommand::List => {
                let mut statement = connection
                    .prepare("SELECT * FROM actions")
                    .map_err(|e| format!("unable to prepare statement: {}", e))?;
                let mut rows = statement
                    .query([])
                    .map_err(|e| format!("unable to execute statement: {}", e))?;
                while let Some(row) = rows
                    .next()
                    .map_err(|e| format!("unable to read row: {}", e))?
                {
                    let description: String = row
                        .get(0)
                        .map_err(|e| format!("unable to read description: {}", e))?;
                    println!("{}", description);
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
                    "rm" => {
                        let args = args.collect::<Vec<_>>();
                        if args.is_empty() {
                            return Err("missing description".into());
                        }
                        Ok(Command::Action(ActionSubcommand::Remove {
                            description: args.join(" "),
                        }))
                    }
                    "ls" => Ok(Command::Action(ActionSubcommand::List)),
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
}

enum ActionSubcommand {
    Add { description: String },
    Remove { description: String },
    List,
}
