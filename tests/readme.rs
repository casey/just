use crate::common::*;

use std::{fs, process::Command};

use executable_path::executable_path;
use test_utilities::assert_success;

#[test]
fn readme() {
  let mut justfiles = vec![];
  let mut current = None;

  for line in fs::read_to_string("README.adoc").unwrap().lines() {
    if let Some(mut justfile) = current {
      if line == "```" {
        justfiles.push(justfile);
        current = None;
      } else {
        justfile += line;
        justfile += "\n";
        current = Some(justfile);
      }
    } else if line == "```make" {
      current = Some(String::new());
    }
  }

  for justfile in justfiles {
    let tmp = tempdir();

    let path = tmp.path().join("justfile");

    fs::write(&path, &justfile).unwrap();

    let output = Command::new(executable_path("just"))
      .current_dir(tmp.path())
      .arg("--dump")
      .output()
      .unwrap();

    assert_success(&output);
  }
}
