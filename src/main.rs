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
#[cfg(all(unix, not(target_os = "macos")))]
use std::os::unix::fs::DirBuilderExt;
use std::{env, fs::DirBuilder, process};

fn main() {
    if let Err(e) = run() {
        eprintln!("odo: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    validate_args()?;
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
    Ok(())
}

fn validate_args() -> Result<(), String> {
    let mut args = env::args().skip(1);
    match args.next() {
        Some(subcommand) => match subcommand.as_str() {
            "action" => match args.next() {
                Some(subsubcommand) => match subsubcommand.as_str() {
                    "add" => {
                        if args.next().is_none() {
                            return Err("missing description".into());
                        }
                        Ok(())
                    }
                    "ls" => Ok(()),
                    _ => Err(format!("no such subsubcommand: `{}`", subsubcommand)),
                },
                None => Err("missing subsubcommand".into()),
            },
            _ => Err(format!("no such subcommand: `{}`", subcommand)),
        },
        None => Err("missing subcommand".into()),
    }
}
