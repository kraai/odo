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
    if let Some(subcommand) = env::args().nth(1) {
        if subcommand == "action" {
            if let Some(subsubcommand) = env::args().nth(2) {
                eprintln!("odo: no such subsubcommand: `{}`", subsubcommand);
            } else {
                eprintln!("odo: missing subsubcommand");
            }
        } else {
            eprintln!("odo: no such subcommand: `{}`", subcommand);
        }
    } else {
        eprintln!("odo: missing subcommand");
    }
    process::exit(1);
}
