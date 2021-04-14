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
#[cfg(all(unix, not(target_os = "macos")))]
use std::os::unix::fs::MetadataExt;
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
fn missing_command_does_not_create_data_directory() {
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
    assert!(!data_dir.is_dir());
}

#[test]
fn reports_missing_command() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: missing command\n");
}

#[test]
fn reports_no_such_command() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .arg("foo")
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: no such command: `foo`\n");
}

#[test]
fn reports_missing_action_subcommand() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .arg("action")
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: missing subcommand\n");
}

#[test]
fn reports_no_such_action_subcommand() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["action", "foo"])
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: no such subcommand: `foo`\n");
}

#[test]
fn reports_missing_action_description() {
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
fn creates_data_directory() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["action", "add", "Read", "*Network", "Effect*."])
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

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn creates_parent_directories_0o700() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["action", "add", "Read", "*Network", "Effect*."])
        .assert();
    assert_eq!(
        home_dir.path().join(".local").metadata().unwrap().mode() & 0o700,
        0o700
    );
    assert_eq!(
        home_dir
            .path()
            .join(".local/share")
            .metadata()
            .unwrap()
            .mode()
            & 0o700,
        0o700
    );
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

#[test]
fn lists_action() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["action", "add", "Read", "*Network", "Effect*."])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["action", "ls"])
        .assert()
        .success()
        .stdout("Read *Network Effect*.\n")
        .stderr("");
}

#[test]
fn removes_action() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["action", "add", "Read", "*Network", "Effect*."])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["action", "rm", "Read", "*Network", "Effect*."])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["action", "ls"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

#[test]
fn reports_missing_goal_subcommand() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .arg("goal")
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: missing subcommand\n");
}

#[test]
fn reports_no_such_goal_subcommand() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["goal", "foo"])
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: no such subcommand: `foo`\n");
}

#[test]
fn reports_missing_goal_description() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["goal", "add"])
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: missing description\n");
}

#[test]
fn adds_goal() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["goal", "add", "Read", "*Network", "Effect*."])
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

#[test]
fn lists_no_goals() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["goal", "ls"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

#[test]
fn lists_goal() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["goal", "add", "Read", "*Network", "Effect*."])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["goal", "ls"])
        .assert()
        .success()
        .stdout("Read *Network Effect*.\n")
        .stderr("");
}

#[test]
fn removes_goal() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["goal", "add", "Read", "*Network", "Effect*."])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["goal", "rm", "Read", "*Network", "Effect*."])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["goal", "ls"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
}
