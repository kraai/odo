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
    if let Err(e) = run() {
        eprintln!("odo: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    match args.next() {
        Some(subcommand) => match subcommand.as_str() {
            "action" => match args.next() {
                Some(subsubcommand) => match subsubcommand.as_str() {
                    "add" => {
                        if args.next().is_none() {
                            return Err("missing description".into());
                        }
                    }
                    _ => return Err(format!("no such subsubcommand: `{}`", subsubcommand)),
                },
                None => return Err("missing subsubcommand".into()),
            },
            _ => return Err(format!("no such subcommand: `{}`", subcommand)),
        },
        None => return Err("missing subcommand".into()),
    }
    Ok(())
}
