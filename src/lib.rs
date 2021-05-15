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
use rusqlite::{config::DbConfig, Connection};
#[cfg(all(unix, not(target_os = "macos")))]
use std::os::unix::fs::DirBuilderExt;
use std::{
    fs::DirBuilder,
    io::{self, Write},
};

pub fn run<T: Iterator<Item = String>>(args: T) -> Result<(), String> {
    let command = parse_args(args)?;
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
    initialize(&connection).map_err(|e| format!("unable to initialize database: {}", e))?;
    match command {
        Command::Action(subcommand) => match subcommand {
            ActionSubcommand::Add { description } => add_action(&connection, description)?,
            ActionSubcommand::List => list_actions(&connection, &mut io::stdout())?,
            ActionSubcommand::Remove { description } => remove_action(&connection, description)?,
            ActionSubcommand::SetDescription {
                old_description,
                new_description,
            } => set_action_description(&connection, old_description, new_description)?,
        },
        Command::Goal(subcommand) => match subcommand {
            GoalSubcommand::Add {
                description,
                action,
            } => add_goal(&connection, description, action)?,
            GoalSubcommand::List { all } => list_goals(&connection, all, &mut io::stdout())?,
            GoalSubcommand::Remove { description } => remove_goal(&connection, description)?,
            GoalSubcommand::SetAction {
                description,
                action,
            } => set_goal_action(&connection, description, action)?,
            GoalSubcommand::SetDescription {
                old_description,
                new_description,
            } => set_goal_description(&connection, old_description, new_description)?,
            GoalSubcommand::UnsetAction { description } => {
                unset_goal_action(&connection, description)?
            }
        },
    }
    Ok(())
}

fn parse_args<T: Iterator<Item = String>>(mut args: T) -> Result<Command, String> {
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
                    "ls" => {
                        if let Some(arg) = args.next() {
                            return Err(format!("extra argument: `{}`", arg));
                        }
                        Ok(Command::Action(ActionSubcommand::List))
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
                    "set" => match args.next() {
                        Some(field) => match field.as_str() {
                            "description" => {
                                let old_description = args
                                    .next()
                                    .ok_or_else(|| "missing old description".to_string())?;
                                let args = args.collect::<Vec<_>>();
                                if args.is_empty() {
                                    return Err("missing new description".into());
                                }
                                Ok(Command::Action(ActionSubcommand::SetDescription {
                                    old_description,
                                    new_description: args.join(" "),
                                }))
                            }
                            _ => Err(format!("no such field: `{}`", field)),
                        },
                        None => Err("missing field".into()),
                    },
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
                    "ls" => {
                        let mut all = false;
                        while let Some(arg) = args.next() {
                            if arg == "--all" {
                                all = true;
                            } else {
                                return Err(format!("extra argument: `{}`", arg));
                            }
                        }
                        Ok(Command::Goal(GoalSubcommand::List { all }))
                    }
                    "rm" => {
                        let args = args.collect::<Vec<_>>();
                        if args.is_empty() {
                            return Err("missing description".into());
                        }
                        Ok(Command::Goal(GoalSubcommand::Remove {
                            description: args.join(" "),
                        }))
                    }
                    "set" => match args.next() {
                        Some(field) => match field.as_str() {
                            "action" => {
                                let description = args
                                    .next()
                                    .ok_or_else(|| "missing description".to_string())?;
                                let args = args.collect::<Vec<_>>();
                                if args.is_empty() {
                                    return Err("missing action".into());
                                }
                                Ok(Command::Goal(GoalSubcommand::SetAction {
                                    description,
                                    action: args.join(" "),
                                }))
                            }
                            "description" => {
                                let old_description = args
                                    .next()
                                    .ok_or_else(|| "missing old description".to_string())?;
                                let args = args.collect::<Vec<_>>();
                                if args.is_empty() {
                                    return Err("missing new description".into());
                                }
                                Ok(Command::Goal(GoalSubcommand::SetDescription {
                                    old_description,
                                    new_description: args.join(" "),
                                }))
                            }
                            _ => Err(format!("no such field: `{}`", field)),
                        },
                        None => Err("missing field".into()),
                    },
                    "unset" => match args.next() {
                        Some(field) => match field.as_str() {
                            "action" => {
                                let args = args.collect::<Vec<_>>();
                                if args.is_empty() {
                                    return Err("missing description".into());
                                }
                                Ok(Command::Goal(GoalSubcommand::UnsetAction {
                                    description: args.join(" "),
                                }))
                            }
                            _ => Err(format!("no such field: `{}`", field)),
                        },
                        None => Err("missing field".into()),
                    },
                    _ => Err(format!("no such subcommand: `{}`", subcommand)),
                },
                None => Err("missing subcommand".into()),
            },
            _ => Err(format!("no such command: `{}`", command)),
        },
        None => Err("missing command".into()),
    }
}

#[derive(Debug, PartialEq)]
enum Command {
    Action(ActionSubcommand),
    Goal(GoalSubcommand),
}

#[derive(Debug, PartialEq)]
enum ActionSubcommand {
    Add {
        description: String,
    },
    List,
    Remove {
        description: String,
    },
    SetDescription {
        old_description: String,
        new_description: String,
    },
}

#[derive(Debug, PartialEq)]
enum GoalSubcommand {
    Add {
        description: String,
        action: Option<String>,
    },
    List {
        all: bool,
    },
    Remove {
        description: String,
    },
    SetAction {
        description: String,
        action: String,
    },
    SetDescription {
        old_description: String,
        new_description: String,
    },
    UnsetAction {
        description: String,
    },
}

fn initialize(connection: &Connection) -> Result<(), String> {
    connection
        .set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY, true)
        .map_err(|e| e.to_string())?;
    connection
        .execute_batch(include_str!("initialize.sql"))
        .map_err(|e| e.to_string())
}

fn add_action<T: AsRef<str>>(connection: &Connection, description: T) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO actions VALUES(?1)",
            rusqlite::params![description.as_ref()],
        )
        .map(|_| ())
        .map_err(|e| {
            if let rusqlite::Error::SqliteFailure(
                libsqlite3_sys::Error {
                    code: libsqlite3_sys::ErrorCode::ConstraintViolation,
                    ..
                },
                _,
            ) = e
            {
                "action already exists".into()
            } else {
                format!("unable to add action: {}", e)
            }
        })
}

fn list_actions<T: Write>(connection: &Connection, writer: &mut T) -> Result<(), String> {
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
        writeln!(writer, "{}", description)
            .map_err(|e| format!("unable to write description: {}", e))?;
    }
    Ok(())
}

fn remove_action<T: AsRef<str>>(connection: &Connection, description: T) -> Result<(), String> {
    match connection
        .execute(
            "DELETE FROM actions WHERE description = ?1",
            rusqlite::params![description.as_ref()],
        )
        .map_err(|e| format!("unable to remove action: {}", e))?
    {
        0 => Err("action does not exist".into()),
        1 => Ok(()),
        _ => unreachable!(),
    }
}

fn set_action_description<T: AsRef<str>, U: AsRef<str>>(
    connection: &Connection,
    old_description: T,
    new_description: U,
) -> Result<(), String> {
    match connection
        .execute(
            "UPDATE actions SET description = ?1 WHERE description = ?2",
            rusqlite::params![new_description.as_ref(), old_description.as_ref()],
        )
        .map_err(|e| format!("unable to set description: {}", e))?
    {
        0 => Err("action does not exist".into()),
        1 => Ok(()),
        _ => unreachable!(),
    }
}

fn add_goal<T: AsRef<str>, U: AsRef<str>>(
    connection: &Connection,
    description: T,
    action: Option<U>,
) -> Result<(), String> {
    if let Some(action) = action {
        connection
            .execute(
                "INSERT INTO goals VALUES(?1, ?2)",
                rusqlite::params![description.as_ref(), action.as_ref()],
            )
            .map(|_| ())
            .map_err(|e| {
                if let rusqlite::Error::SqliteFailure(
                    libsqlite3_sys::Error {
                        code: libsqlite3_sys::ErrorCode::ConstraintViolation,
                        ..
                    },
                    _,
                ) = e
                {
                    if e.to_string().starts_with("UNIQUE") {
                        "goal already exists".into()
                    } else {
                        "action does not exist".into()
                    }
                } else {
                    format!("unable to add goal: {}", e)
                }
            })
    } else {
        connection
            .execute(
                "INSERT INTO goals (description) VALUES(?1)",
                rusqlite::params![description.as_ref()],
            )
            .map(|_| ())
            .map_err(|e| {
                if let rusqlite::Error::SqliteFailure(
                    libsqlite3_sys::Error {
                        code: libsqlite3_sys::ErrorCode::ConstraintViolation,
                        ..
                    },
                    _,
                ) = e
                {
                    "goal already exists".into()
                } else {
                    format!("unable to add goal: {}", e)
                }
            })
    }
}

fn list_goals<T: Write>(connection: &Connection, all: bool, writer: &mut T) -> Result<(), String> {
    let statement = if all {
        "SELECT description FROM goals"
    } else {
        "SELECT description FROM goals WHERE action IS NULL"
    };
    let mut statement = connection
        .prepare(statement)
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
        writeln!(writer, "{}", description)
            .map_err(|e| format!("unable to write description: {}", e))?;
    }
    Ok(())
}

fn remove_goal<T: AsRef<str>>(connection: &Connection, description: T) -> Result<(), String> {
    match connection
        .execute(
            "DELETE FROM goals WHERE description = ?1",
            rusqlite::params![description.as_ref()],
        )
        .map_err(|e| format!("unable to remove goal: {}", e))?
    {
        0 => Err("goal does not exist".into()),
        1 => Ok(()),
        _ => unreachable!(),
    }
}

fn set_goal_action<T: AsRef<str>, U: AsRef<str>>(
    connection: &Connection,
    description: T,
    action: U,
) -> Result<(), String> {
    match connection
        .execute(
            "UPDATE goals SET action = ?1 WHERE description = ?2",
            rusqlite::params![action.as_ref(), description.as_ref()],
        )
        .map_err(|e| {
            if let rusqlite::Error::SqliteFailure(
                libsqlite3_sys::Error {
                    code: libsqlite3_sys::ErrorCode::ConstraintViolation,
                    ..
                },
                _,
            ) = e
            {
                "action does not exist".into()
            } else {
                format!("unable to set action: {}", e)
            }
        })? {
        0 => Err("goal does not exist".into()),
        1 => Ok(()),
        _ => unreachable!(),
    }
}

fn set_goal_description<T: AsRef<str>, U: AsRef<str>>(
    connection: &Connection,
    old_description: T,
    new_description: U,
) -> Result<(), String> {
    match connection
        .execute(
            "UPDATE goals SET description = ?1 WHERE description = ?2",
            rusqlite::params![new_description.as_ref(), old_description.as_ref()],
        )
        .map_err(|e| format!("unable to set description: {}", e))?
    {
        0 => Err("goal does not exist".into()),
        1 => Ok(()),
        _ => unreachable!(),
    }
}

fn unset_goal_action<T: AsRef<str>>(connection: &Connection, description: T) -> Result<(), String> {
    match connection
        .execute(
            "UPDATE goals SET action = NULL WHERE description = ?1",
            rusqlite::params![description.as_ref()],
        )
        .map_err(|e| format!("unable to unset action: {}", e))?
    {
        0 => Err("goal does not exist".into()),
        1 => Ok(()),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Error;
    use std::array::IntoIter;

    #[test]
    fn reports_missing_command() {
        assert_eq!(
            parse_args(IntoIter::new([])),
            Err("missing command".to_string())
        );
    }

    #[test]
    fn reports_no_such_command() {
        assert_eq!(
            parse_args(IntoIter::new(["foo".to_string()])),
            Err("no such command: `foo`".to_string())
        );
    }

    #[test]
    fn reports_missing_action_subcommand() {
        assert_eq!(
            parse_args(IntoIter::new(["action".to_string()])),
            Err("missing subcommand".to_string())
        );
    }

    #[test]
    fn reports_no_such_action_subcommand() {
        assert_eq!(
            parse_args(IntoIter::new(["action".to_string(), "foo".to_string()])),
            Err("no such subcommand: `foo`".to_string())
        );
    }

    #[test]
    fn reports_missing_action_add_description() {
        assert_eq!(
            parse_args(IntoIter::new(["action".to_string(), "add".to_string()])),
            Err("missing description".to_string())
        );
    }

    #[test]
    fn reports_extra_action_ls_argument() {
        assert_eq!(
            parse_args(IntoIter::new([
                "action".to_string(),
                "ls".to_string(),
                "foo".to_string()
            ])),
            Err("extra argument: `foo`".to_string())
        );
    }

    #[test]
    fn reports_missing_action_rm_description() {
        assert_eq!(
            parse_args(IntoIter::new(["action".to_string(), "rm".to_string()])),
            Err("missing description".to_string())
        );
    }

    #[test]
    fn reports_missing_action_set_field() {
        assert_eq!(
            parse_args(IntoIter::new(["action".to_string(), "set".to_string()])),
            Err("missing field".to_string())
        );
    }

    #[test]
    fn reports_no_such_action_field() {
        assert_eq!(
            parse_args(IntoIter::new([
                "action".to_string(),
                "set".to_string(),
                "foo".to_string(),
            ])),
            Err("no such field: `foo`".to_string())
        );
    }

    #[test]
    fn reports_missing_old_action_description() {
        assert_eq!(
            parse_args(IntoIter::new([
                "action".to_string(),
                "set".to_string(),
                "description".to_string(),
            ])),
            Err("missing old description".to_string())
        );
    }

    #[test]
    fn reports_missing_new_action_description() {
        assert_eq!(
            parse_args(IntoIter::new([
                "action".to_string(),
                "set".to_string(),
                "description".to_string(),
                "Read *Network Efect*.".to_string(),
            ])),
            Err("missing new description".to_string())
        );
    }

    #[test]
    fn parses_action_set_description() {
        assert_eq!(
            parse_args(IntoIter::new([
                "action".to_string(),
                "set".to_string(),
                "description".to_string(),
                "Read *Network Efect*.".to_string(),
                "Read".to_string(),
                "*Network".to_string(),
                "Effect*.".to_string(),
            ])),
            Ok(Command::Action(ActionSubcommand::SetDescription {
                old_description: "Read *Network Efect*.".into(),
                new_description: "Read *Network Effect*.".into()
            }))
        );
    }

    #[test]
    fn reports_missing_goal_subcommand() {
        assert_eq!(
            parse_args(IntoIter::new(["goal".to_string()])),
            Err("missing subcommand".to_string())
        );
    }

    #[test]
    fn reports_no_such_goal_subcommand() {
        assert_eq!(
            parse_args(IntoIter::new(["goal".to_string(), "foo".to_string()])),
            Err("no such subcommand: `foo`".to_string())
        );
    }

    #[test]
    fn reports_missing_goal_add_description() {
        assert_eq!(
            parse_args(IntoIter::new(["goal".to_string(), "add".to_string()])),
            Err("missing description".to_string())
        );
    }

    #[test]
    fn reports_missing_goal_action() {
        assert_eq!(
            parse_args(IntoIter::new([
                "goal".to_string(),
                "add".to_string(),
                "--action".to_string()
            ])),
            Err("option `--action` requires an argument".to_string())
        );
    }

    #[test]
    fn reports_extra_goal_ls_argument() {
        assert_eq!(
            parse_args(IntoIter::new([
                "goal".to_string(),
                "ls".to_string(),
                "foo".to_string()
            ])),
            Err("extra argument: `foo`".to_string())
        );
    }

    #[test]
    fn reports_extra_goal_ls_argument_after_all() {
        assert_eq!(
            parse_args(IntoIter::new([
                "goal".to_string(),
                "ls".to_string(),
                "--all".to_string(),
                "foo".to_string()
            ])),
            Err("extra argument: `foo`".to_string())
        );
    }

    #[test]
    fn parses_goal_ls() {
        assert_eq!(
            parse_args(IntoIter::new(["goal".to_string(), "ls".to_string()])),
            Ok(Command::Goal(GoalSubcommand::List { all: false }))
        );
    }

    #[test]
    fn parses_goal_ls_all() {
        assert_eq!(
            parse_args(IntoIter::new([
                "goal".to_string(),
                "ls".to_string(),
                "--all".to_string()
            ])),
            Ok(Command::Goal(GoalSubcommand::List { all: true }))
        );
    }

    #[test]
    fn reports_missing_goal_rm_description() {
        assert_eq!(
            parse_args(IntoIter::new(["goal".to_string(), "rm".to_string()])),
            Err("missing description".to_string())
        );
    }

    #[test]
    fn reports_missing_goal_set_field() {
        assert_eq!(
            parse_args(IntoIter::new(["goal".to_string(), "set".to_string()])),
            Err("missing field".to_string())
        );
    }

    #[test]
    fn reports_no_such_goal_set_field() {
        assert_eq!(
            parse_args(IntoIter::new([
                "goal".to_string(),
                "set".to_string(),
                "foo".to_string(),
            ])),
            Err("no such field: `foo`".to_string())
        );
    }

    #[test]
    fn reports_missing_goal_set_description() {
        assert_eq!(
            parse_args(IntoIter::new([
                "goal".to_string(),
                "set".to_string(),
                "action".to_string(),
            ])),
            Err("missing description".to_string())
        );
    }

    #[test]
    fn reports_missing_action() {
        assert_eq!(
            parse_args(IntoIter::new([
                "goal".to_string(),
                "set".to_string(),
                "action".to_string(),
                "Read *Network Effect*.".to_string(),
            ])),
            Err("missing action".to_string())
        );
    }

    #[test]
    fn parses_goal_set_action() {
        assert_eq!(
            parse_args(IntoIter::new([
                "goal".to_string(),
                "set".to_string(),
                "action".to_string(),
                "Read *Network Effect*.".to_string(),
                "Borrow".to_string(),
                "*Network".to_string(),
                "Effect*.".to_string(),
            ])),
            Ok(Command::Goal(GoalSubcommand::SetAction {
                description: "Read *Network Effect*.".into(),
                action: "Borrow *Network Effect*.".into()
            }))
        );
    }

    #[test]
    fn reports_missing_old_goal_description() {
        assert_eq!(
            parse_args(IntoIter::new([
                "goal".to_string(),
                "set".to_string(),
                "description".to_string(),
            ])),
            Err("missing old description".to_string())
        );
    }

    #[test]
    fn reports_missing_new_goal_description() {
        assert_eq!(
            parse_args(IntoIter::new([
                "goal".to_string(),
                "set".to_string(),
                "description".to_string(),
                "Read *Network Efect*.".to_string(),
            ])),
            Err("missing new description".to_string())
        );
    }

    #[test]
    fn parses_goal_set_description() {
        assert_eq!(
            parse_args(IntoIter::new([
                "goal".to_string(),
                "set".to_string(),
                "description".to_string(),
                "Read *Network Efect*.".to_string(),
                "Read".to_string(),
                "*Network".to_string(),
                "Effect*.".to_string(),
            ])),
            Ok(Command::Goal(GoalSubcommand::SetDescription {
                old_description: "Read *Network Efect*.".into(),
                new_description: "Read *Network Effect*.".into()
            }))
        );
    }

    #[test]
    fn reports_missing_goal_unset_field() {
        assert_eq!(
            parse_args(IntoIter::new(["goal".to_string(), "unset".to_string()])),
            Err("missing field".to_string())
        );
    }

    #[test]
    fn reports_no_such_goal_unset_field() {
        assert_eq!(
            parse_args(IntoIter::new([
                "goal".to_string(),
                "unset".to_string(),
                "foo".to_string(),
            ])),
            Err("no such field: `foo`".to_string())
        );
    }

    #[test]
    fn reports_missing_goal_unset_description() {
        assert_eq!(
            parse_args(IntoIter::new([
                "goal".to_string(),
                "unset".to_string(),
                "action".to_string(),
            ])),
            Err("missing description".to_string())
        );
    }

    #[test]
    fn parses_goal_unset_action() {
        assert_eq!(
            parse_args(IntoIter::new([
                "goal".to_string(),
                "unset".to_string(),
                "action".to_string(),
                "Read".to_string(),
                "*Network".to_string(),
                "Effect*.".to_string(),
            ])),
            Ok(Command::Goal(GoalSubcommand::UnsetAction {
                description: "Read *Network Effect*.".into(),
            }))
        );
    }

    #[test]
    fn adds_action() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        add_action(&connection, "Read *Network Effect*.").unwrap();
        assert_eq!(
            connection
                .query_row("SELECT * FROM actions", [], |row| row
                    .get::<usize, String>(0))
                .unwrap(),
            "Read *Network Effect*."
        );
    }

    #[test]
    fn fails_to_add_duplicate_action() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        add_action(&connection, "Read *Network Effect*.").unwrap();
        assert_eq!(
            add_action(&connection, "Read *Network Effect*."),
            Err("action already exists".to_string())
        );
    }

    #[test]
    fn lists_no_actions() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        let mut output = Vec::new();
        list_actions(&connection, &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "");
    }

    #[test]
    fn lists_action() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        connection
            .execute("INSERT INTO actions VALUES('Read *Network Effect*.')", [])
            .unwrap();
        let mut output = Vec::new();
        list_actions(&connection, &mut output).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "Read *Network Effect*.\n"
        );
    }

    #[test]
    fn lists_actions() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        connection
            .execute("INSERT INTO actions VALUES('Read *Network Effect*.')", [])
            .unwrap();
        connection
            .execute(
                "INSERT INTO actions VALUES('Read *What Were We Thinking*.')",
                [],
            )
            .unwrap();
        let mut output = Vec::new();
        list_actions(&connection, &mut output).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "Read *Network Effect*.\nRead *What Were We Thinking*.\n"
        );
    }

    #[test]
    fn removes_action() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        connection
            .execute("INSERT INTO actions VALUES('Read *Network Effect*.')", [])
            .unwrap();
        remove_action(&connection, "Read *Network Effect*.").unwrap();
        assert_eq!(
            connection.query_row("SELECT * FROM actions", [], |_| Ok(())),
            Err(Error::QueryReturnedNoRows)
        );
    }

    #[test]
    fn fails_to_remove_nonexistent_action() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        assert_eq!(
            remove_action(&connection, "Read *Network Effect*."),
            Err("action does not exist".to_string())
        );
    }

    #[test]
    fn removing_action_clears_goal_action() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        connection
            .execute("INSERT INTO actions VALUES('Borrow *Network Effect*.')", [])
            .unwrap();
        connection
            .execute(
                "INSERT INTO goals VALUES('Read *Network Effect*.', 'Borrow *Network Effect*.')",
                [],
            )
            .unwrap();
        remove_action(&connection, "Borrow *Network Effect*.").unwrap();
        assert_eq!(
            connection
                .query_row("SELECT action FROM goals", [], |row| row
                    .get::<usize, Option<String>>(0))
                .unwrap(),
            None
        );
    }

    #[test]
    fn sets_action_description() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        connection
            .execute(
                "INSERT INTO actions (description) VALUES('Read *Network Efect*.')",
                [],
            )
            .unwrap();
        set_action_description(
            &connection,
            "Read *Network Efect*.",
            "Read *Network Effect*.",
        )
        .unwrap();
        assert_eq!(
            connection
                .query_row("SELECT description FROM actions", [], |row| row
                    .get::<usize, String>(0))
                .unwrap(),
            "Read *Network Effect*."
        );
    }

    #[test]
    fn fails_to_set_nonexistent_action_description() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        assert_eq!(
            set_action_description(
                &connection,
                "Read *Network Efect*.",
                "Read *Network Effect*."
            ),
            Err("action does not exist".to_string())
        );
    }

    #[test]
    fn updates_goal_action() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        connection
            .execute("INSERT INTO actions VALUES('Borrow *Network Efect*.')", [])
            .unwrap();
        connection
            .execute(
                "INSERT INTO goals VALUES('Read *Network Effect*.', 'Borrow *Network Efect*.')",
                [],
            )
            .unwrap();
        set_action_description(
            &connection,
            "Borrow *Network Efect*.",
            "Borrow *Network Effect*.",
        )
        .unwrap();
        assert_eq!(
            connection
                .query_row("SELECT action FROM goals", [], |row| row
                    .get::<usize, String>(0))
                .unwrap(),
            "Borrow *Network Effect*.",
        );
    }

    #[test]
    fn adds_goal() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        add_goal::<&str, &str>(&connection, "Read *Network Effect*.", None).unwrap();
        let (description, action): (String, Option<String>) = connection
            .query_row("SELECT * FROM goals", [], |row| {
                Ok((row.get_unwrap(0), row.get_unwrap(1)))
            })
            .unwrap();
        assert_eq!(description, "Read *Network Effect*.");
        assert_eq!(action, None);
    }

    #[test]
    fn adds_goal_with_action() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        connection
            .execute("INSERT INTO actions VALUES('Borrow *Network Effect*.')", [])
            .unwrap();
        add_goal(
            &connection,
            "Read *Network Effect*.",
            Some("Borrow *Network Effect*."),
        )
        .unwrap();
        let (description, action): (String, Option<String>) = connection
            .query_row("SELECT * FROM goals", [], |row| {
                Ok((row.get_unwrap(0), row.get_unwrap(1)))
            })
            .unwrap();
        assert_eq!(description, "Read *Network Effect*.");
        assert_eq!(action, Some("Borrow *Network Effect*.".to_string()));
    }

    #[test]
    fn fails_to_add_duplicate_goal() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        add_goal::<&str, &str>(&connection, "Read *Network Effect*.", None).unwrap();
        assert_eq!(
            add_goal::<&str, &str>(&connection, "Read *Network Effect*.", None),
            Err("goal already exists".to_string())
        );
    }

    #[test]
    fn fails_to_add_duplicate_goal_with_action() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        add_action(&connection, "Borrow *Network Effect*.").unwrap();
        add_goal::<&str, &str>(&connection, "Read *Network Effect*.", None).unwrap();
        assert_eq!(
            add_goal(
                &connection,
                "Read *Network Effect*.",
                Some("Borrow *Network Effect*.")
            ),
            Err("goal already exists".to_string())
        );
    }

    #[test]
    fn fails_to_add_goal_with_nonexistent_action() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        assert_eq!(
            add_goal(
                &connection,
                "Read *Network Effect*.",
                Some("Borrow *Network Effect*."),
            ),
            Err("action does not exist".to_string())
        );
    }

    #[test]
    fn lists_no_goals() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        let mut output = Vec::new();
        list_goals(&connection, false, &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "");
    }

    #[test]
    fn lists_goal() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        connection
            .execute(
                "INSERT INTO goals (description) VALUES('Read *Network Effect*.')",
                [],
            )
            .unwrap();
        let mut output = Vec::new();
        list_goals(&connection, false, &mut output).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "Read *Network Effect*.\n"
        );
    }

    #[test]
    fn lists_goals() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        connection
            .execute(
                "INSERT INTO goals (description) VALUES('Read *Network Effect*.')",
                [],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO goals (description) VALUES('Read *What Were We Thinking*.')",
                [],
            )
            .unwrap();
        let mut output = Vec::new();
        list_goals(&connection, false, &mut output).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "Read *Network Effect*.\nRead *What Were We Thinking*.\n"
        );
    }

    #[test]
    fn does_not_list_goal_with_action() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        connection
            .execute("INSERT INTO actions VALUES('Borrow *Network Effect*.')", [])
            .unwrap();
        connection
            .execute(
                "INSERT INTO goals VALUES('Read *Network Effect*.', 'Borrow *Network Effect*.')",
                [],
            )
            .unwrap();
        let mut output = Vec::new();
        list_goals(&connection, false, &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "");
    }

    #[test]
    fn lists_goal_with_action() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        connection
            .execute("INSERT INTO actions VALUES('Borrow *Network Effect*.')", [])
            .unwrap();
        connection
            .execute(
                "INSERT INTO goals VALUES('Read *Network Effect*.', 'Borrow *Network Effect*.')",
                [],
            )
            .unwrap();
        let mut output = Vec::new();
        list_goals(&connection, true, &mut output).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "Read *Network Effect*.\n"
        );
    }

    #[test]
    fn removes_goal() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        connection
            .execute(
                "INSERT INTO goals (description) VALUES('Read *Network Effect*.')",
                [],
            )
            .unwrap();
        remove_goal(&connection, "Read *Network Effect*.").unwrap();
        assert_eq!(
            connection.query_row("SELECT * FROM goals", [], |_| Ok(())),
            Err(Error::QueryReturnedNoRows)
        );
    }

    #[test]
    fn fails_to_remove_nonexistent_goal() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        assert_eq!(
            remove_goal(&connection, "Read *Network Effect*."),
            Err("goal does not exist".to_string())
        );
    }

    #[test]
    fn sets_goal_action() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        connection
            .execute("INSERT INTO actions VALUES('Borrow *Network Effect*.')", [])
            .unwrap();
        connection
            .execute(
                "INSERT INTO goals (description) VALUES('Read *Network Effect*.')",
                [],
            )
            .unwrap();
        set_goal_action(
            &connection,
            "Read *Network Effect*.",
            "Borrow *Network Effect*.",
        )
        .unwrap();
        assert_eq!(
            connection
                .query_row("SELECT action FROM goals", [], |row| row
                    .get::<usize, String>(0))
                .unwrap(),
            "Borrow *Network Effect*."
        );
    }

    #[test]
    fn fails_to_set_nonexistent_goal_action() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        assert_eq!(
            set_goal_description(
                &connection,
                "Read *Network Effect*.",
                "Borrow *Network Effect*."
            ),
            Err("goal does not exist".to_string())
        );
    }

    #[test]
    fn sets_goal_description() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        connection
            .execute(
                "INSERT INTO goals (description) VALUES('Read *Network Efect*.')",
                [],
            )
            .unwrap();
        set_goal_description(
            &connection,
            "Read *Network Efect*.",
            "Read *Network Effect*.",
        )
        .unwrap();
        assert_eq!(
            connection
                .query_row("SELECT description FROM goals", [], |row| row
                    .get::<usize, String>(0))
                .unwrap(),
            "Read *Network Effect*."
        );
    }

    #[test]
    fn fails_to_set_nonexistent_goal_description() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        assert_eq!(
            set_goal_description(
                &connection,
                "Read *Network Efect*.",
                "Read *Network Effect*."
            ),
            Err("goal does not exist".to_string())
        );
    }

    #[test]
    fn unsets_goal_action() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        connection
            .execute("INSERT INTO actions VALUES('Borrow *Network Effect*.')", [])
            .unwrap();
        connection
            .execute(
                "INSERT INTO goals VALUES('Read *Network Effect*.', 'Borrow *Network Effect*.')",
                [],
            )
            .unwrap();
        unset_goal_action(&connection, "Read *Network Effect*.").unwrap();
        assert_eq!(
            connection
                .query_row("SELECT action FROM goals", [], |row| row
                    .get::<usize, Option<String>>(0))
                .unwrap(),
            None
        );
    }

    #[test]
    fn fails_to_unset_action_of_nonexistent_goal() {
        let connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        assert_eq!(
            unset_goal_action(&connection, "Read *Network Effect*."),
            Err("goal does not exist".to_string())
        );
    }
}
