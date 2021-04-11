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

#[test]
fn reports_missing_subcommand() {
    Command::cargo_bin("odo")
        .unwrap()
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: missing subcommand\n");
}

#[test]
fn reports_no_such_subcommand() {
    Command::cargo_bin("odo")
        .unwrap()
        .arg("foo")
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: no such subcommand: `foo`\n");
}

#[test]
fn reports_missing_subsubcommand() {
    Command::cargo_bin("odo")
        .unwrap()
        .arg("action")
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: missing subsubcommand\n");
}

#[test]
fn reports_no_such_subsubcommand() {
    Command::cargo_bin("odo")
        .unwrap()
        .args(&["action", "foo"])
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: no such subsubcommand: `foo`\n");
}

#[test]
fn reports_missing_description() {
    Command::cargo_bin("odo")
        .unwrap()
        .args(&["action", "add"])
        .assert()
        .failure()
        .stdout("")
        .stderr("odo: missing description\n");
}

#[test]
fn adds_action() {
    Command::cargo_bin("odo")
        .unwrap()
        .args(&["action", "add", "Read", "*Network", "Effect*."])
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

#[test]
fn lists_no_actions() {
    Command::cargo_bin("odo")
        .unwrap()
        .args(&["action", "ls"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
}
