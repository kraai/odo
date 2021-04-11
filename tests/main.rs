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

use assert_cmd::Command;
use std::{ffi::OsStr, fs, path::Path};
use tempfile::TempDir;

struct TempHomeDir {
    dir: TempDir,
}

impl TempHomeDir {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        if cfg!(windows) {
            fs::create_dir_all(dir.path().join("AppData").join("Local")).unwrap();
            fs::create_dir_all(dir.path().join("AppData").join("Roaming")).unwrap();
        }
        TempHomeDir { dir }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }
}

trait CommandExt {
    fn home_dir<P: AsRef<Path> + AsRef<OsStr>>(&mut self, dir: P) -> &mut Self;
}

impl CommandExt for Command {
    fn home_dir<P: AsRef<Path> + AsRef<OsStr>>(&mut self, dir: P) -> &mut Command {
        if cfg!(target_os = "macos") {
            self.env("HOME", dir)
        } else if cfg!(unix) {
            self.env("HOME", dir).env_remove("XDG_DATA_DIR")
        } else if cfg!(windows) {
            self.env("USERPROFILE", dir)
        } else {
            unimplemented!()
        }
    }
}

#[test]
fn creates_data_directory() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .assert();
    let data_dir = if cfg!(target_os = "macos") {
        home_dir
            .path()
            .join("Library/Application Support/org.ftbfs.odo")
    } else if cfg!(unix) {
        home_dir.path().join(".local/share/odo")
    } else if cfg!(windows) {
        home_dir.path().join("AppData\\Roaming\\odo")
    } else {
        unimplemented!()
    };
    assert!(data_dir.is_dir());
}

#[test]
fn reports_missing_subcommand() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: missing subcommand\n");
}

#[test]
fn reports_no_such_subcommand() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .arg("foo")
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: no such subcommand: `foo`\n");
}

#[test]
fn reports_missing_subsubcommand() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .arg("action")
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: missing subsubcommand\n");
}

#[test]
fn reports_no_such_subsubcommand() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["action", "foo"])
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: no such subsubcommand: `foo`\n");
}

#[test]
fn reports_missing_description() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["action", "add"])
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: missing description\n");
}

#[test]
fn adds_action() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["action", "add", "Read", "*Network", "Effect*."])
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

#[test]
fn lists_no_actions() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["action", "ls"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
}
