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
use std::fs::DirBuilder;
#[cfg(all(unix, not(target_os = "macos")))]
use std::os::unix::fs::DirBuilderExt;

pub fn open() -> Result<Connection, String> {
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
    Connection::open(&database_path)
        .map_err(|e| format!("unable to open `{}`: {}", database_path.display(), e))
}

pub fn initialize(connection: &Connection) -> Result<(), String> {
    connection
        .set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY, true)
        .map_err(|e| e.to_string())?;
    connection
        .execute_batch(include_str!("initialize.sql"))
        .map_err(|e| e.to_string())
}
