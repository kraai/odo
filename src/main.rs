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

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use diesel::{prelude::*, Connection, SqliteConnection};
use directories::ProjectDirs;
#[cfg(all(unix, not(target_os = "macos")))]
use std::os::unix::fs::DirBuilderExt;
use std::{env, fs::DirBuilder, process};

table! {
    actions (description) {
    description -> Text,
    }
}

embed_migrations!("migrations");

fn main() {
    if let Err(e) = run() {
        eprintln!("odo: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let subcommand = parse_args()?;
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
    let connection = SqliteConnection::establish(
        database_path
            .to_str()
            .ok_or_else(|| format!("unable to convert `{}` to UTF-8", database_path.display()))?,
    )
    .map_err(|e| format!("unable to open `{}`: {}", database_path.display(), e))?;
    embedded_migrations::run(&connection)
        .map_err(|e| format!("unable to run migrations: {}", e))?;
    match subcommand {
        Subcommand::Action(subsubcommand) => match subsubcommand {
            ActionSubcommand::Add { description } => {
                diesel::insert_into(actions::table)
                    .values(&Action { description })
                    .execute(&connection)
                    .map_err(|e| format!("unable to add action: {}", e))?;
            }
            ActionSubcommand::List => {
                let results = actions::table
                    .load::<Action>(&connection)
                    .map_err(|e| format!("unable to load actions: {}", e))?;

                for action in results {
                    println!("{}", action.description);
                }
            }
        },
    }
    Ok(())
}

fn parse_args() -> Result<Subcommand, String> {
    let mut args = env::args().skip(1);
    match args.next() {
        Some(subcommand) => match subcommand.as_str() {
            "action" => match args.next() {
                Some(subsubcommand) => match subsubcommand.as_str() {
                    "add" => {
                        let args = args.collect::<Vec<_>>();
                        if args.is_empty() {
                            return Err("missing description".into());
                        }
                        Ok(Subcommand::Action(ActionSubcommand::Add {
                            description: args.join(" "),
                        }))
                    }
                    "ls" => Ok(Subcommand::Action(ActionSubcommand::List)),
                    _ => Err(format!("no such subsubcommand: `{}`", subsubcommand)),
                },
                None => Err("missing subsubcommand".into()),
            },
            _ => Err(format!("no such subcommand: `{}`", subcommand)),
        },
        None => Err("missing subcommand".into()),
    }
}

enum Subcommand {
    Action(ActionSubcommand),
}

enum ActionSubcommand {
    Add { description: String },
    List,
}

#[derive(Insertable, Queryable)]
#[table_name = "actions"]
struct Action {
    description: String,
}
