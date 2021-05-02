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

use rusqlite::{config::DbConfig, Connection};
use std::io::Write;

pub fn initialize(connection: &Connection) -> Result<(), String> {
    connection
        .set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY, true)
        .map_err(|e| e.to_string())?;
    connection
        .execute_batch(include_str!("initialize.sql"))
        .map_err(|e| e.to_string())
}

pub fn add_action<T: AsRef<str>>(connection: &Connection, description: T) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO actions VALUES(?1)",
            rusqlite::params![description.as_ref()],
        )
        .map(|_| ())
        .map_err(|e| format!("unable to add action: {}", e))
}

pub fn list_actions<T: Write>(connection: &Connection, writer: &mut T) -> Result<(), String> {
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

pub fn remove_action<T: AsRef<str>>(connection: &Connection, description: T) -> Result<(), String> {
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

pub fn add_goal<T: AsRef<str>, U: AsRef<str>>(
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
                    "action does not exist".into()
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
            .map_err(|e| format!("unable to add goal: {}", e))
    }
}

pub fn list_goals<T: Write>(connection: &Connection, writer: &mut T) -> Result<(), String> {
    let mut statement = connection
        .prepare("SELECT * FROM goals WHERE action IS NULL")
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Error;

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
    fn clears_goal_action() {
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
        list_goals(&connection, &mut output).unwrap();
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
        list_goals(&connection, &mut output).unwrap();
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
        list_goals(&connection, &mut output).unwrap();
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
        list_goals(&connection, &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "");
    }
}
