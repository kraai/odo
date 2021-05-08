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
fn odo_action_add_adds_action() {
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
fn odo_action_add_creates_data_directory() {
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
fn odo_action_add_creates_parent_directories_0o700() {
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
fn odo_action_ls_lists_action() {
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
fn odo_action_rm_removes_action() {
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
fn odo_action_set_description_sets_description() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["action", "add", "Read", "*Network", "Efect*."])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&[
            "action",
            "set",
            "description",
            "Read *Network Efect*.",
            "Read",
            "*Network",
            "Effect*.",
        ])
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
fn odo_goal_add_adds_goal() {
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
fn odo_goal_ls_lists_goal() {
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
fn odo_goal_rm_removes_goal() {
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

#[test]
fn odo_goal_set_description_sets_action() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["action", "add", "Borrow", "*Network", "Effect*."])
        .assert()
        .success()
        .stdout("")
        .stderr("");
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
        .args(&[
            "goal",
            "set",
            "action",
            "Read *Network Effect*.",
            "Borrow",
            "*Network",
            "Effect*.",
        ])
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

#[test]
fn odo_goal_set_description_sets_description() {
    let home_dir = TempHomeDir::new();
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&["goal", "add", "Read", "*Network", "Efect*."])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    Command::cargo_bin("odo")
        .unwrap()
        .home_dir(home_dir.path())
        .args(&[
            "goal",
            "set",
            "description",
            "Read *Network Efect*.",
            "Read",
            "*Network",
            "Effect*.",
        ])
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
