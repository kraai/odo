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

use command::Command;

mod command;
mod database;

pub fn run<T: Iterator<Item = String>>(args: T) -> Result<(), String> {
    let command = Command::from_args(args)?;
    let connection = database::open()?;
    database::initialize(&connection)
        .map_err(|e| format!("unable to initialize database: {}", e))?;
    command.run(&connection)
}
