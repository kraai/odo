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

use std::{env, process};

fn main() {
    let mut args = env::args().skip(1);
    match args.next() {
        Some(subcommand) => match subcommand.as_str() {
            "action" => match args.next() {
                Some(subsubcommand) => match subsubcommand.as_str() {
                    "add" => eprintln!("odo: missing description"),
                    _ => eprintln!("odo: no such subsubcommand: `{}`", subsubcommand),
                },
                None => eprintln!("odo: missing subsubcommand"),
            },
            _ => eprintln!("odo: no such subcommand: `{}`", subcommand),
        },
        None => eprintln!("odo: missing subcommand"),
    }
    process::exit(1);
}
