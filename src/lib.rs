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

use rusqlite::Connection;
use std::io::Write;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_action() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(include_str!("initialize.sql"))
            .unwrap();
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
    fn list_nothing() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(include_str!("initialize.sql"))
            .unwrap();
        let mut output = Vec::new();
        list_actions(&connection, &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "");
    }

    #[test]
    fn lists_action() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(include_str!("initialize.sql"))
            .unwrap();
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
        connection
            .execute_batch(include_str!("initialize.sql"))
            .unwrap();
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
}
