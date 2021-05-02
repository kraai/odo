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

pub fn add_action<T: AsRef<str>>(connection: &Connection, description: T) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO actions VALUES(?1)",
            rusqlite::params![description.as_ref()],
        )
        .map(|_| ())
        .map_err(|e| format!("unable to add action: {}", e))
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
}
