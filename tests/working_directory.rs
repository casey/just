use crate::common::*;

use std::{error::Error, process::Command};

use executable_path::executable_path;

const JUSTFILE: &str = r#"
foo := `cat data`

linewise bar=`cat data`: shebang
  echo expression: {{foo}}
  echo default: {{bar}}
  echo linewise: `cat data`

shebang:
  #!/usr/bin/env sh
  echo "shebang:" `cat data`
"#;

const DATA: &str = "OK";

const WANT: &str = "shebang: OK\nexpression: OK\ndefault: OK\nlinewise: OK\n";

/// Test that just runs with the correct working directory when invoked with
/// `--justfile` but not `--working-directory`
#[test]
fn justfile_without_working_directory() -> Result<(), Box<dyn Error>> {
  let tmp = temptree! {
    justfile: JUSTFILE,
    data: DATA,
  };

  let output = Command::new(executable_path("just"))
    .arg("--justfile")
    .arg(&tmp.path().join("justfile"))
    .output()?;

  if !output.status.success() {
    eprintln!("{:?}", String::from_utf8_lossy(&output.stderr));
    panic!();
  }

  let stdout = String::from_utf8(output.stdout).unwrap();

  assert_eq!(stdout, WANT);

  Ok(())
}

/// Test that just runs with the correct working directory when invoked with
/// `--justfile` but not `--working-directory`, and justfile path has no parent
#[test]
fn justfile_without_working_directory_relative() -> Result<(), Box<dyn Error>> {
  let tmp = temptree! {
    justfile: JUSTFILE,
    data: DATA,
  };

  let output = Command::new(executable_path("just"))
    .current_dir(&tmp.path())
    .arg("--justfile")
    .arg("justfile")
    .output()?;

  if !output.status.success() {
    eprintln!("{:?}", String::from_utf8_lossy(&output.stderr));
    panic!();
  }

  let stdout = String::from_utf8(output.stdout).unwrap();

  assert_eq!(stdout, WANT);

  Ok(())
}

/// Test that just invokes commands from the directory in which the justfile is
/// found
#[test]
fn change_working_directory_to_search_justfile_parent() -> Result<(), Box<dyn Error>> {
  let tmp = temptree! {
    justfile: JUSTFILE,
    data: DATA,
    subdir: {},
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path().join("subdir"))
    .output()?;

  if !output.status.success() {
    eprintln!("{:?}", String::from_utf8_lossy(&output.stderr));
    panic!();
  }

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(stdout, WANT);

  Ok(())
}

/// Test that just runs with the correct working directory when invoked with
/// `--justfile` but not `--working-directory`
#[test]
fn justfile_and_working_directory() -> Result<(), Box<dyn Error>> {
  let tmp = temptree! {
    justfile: JUSTFILE,
    sub: {
      data: DATA,
    },
  };

  let output = Command::new(executable_path("just"))
    .arg("--justfile")
    .arg(&tmp.path().join("justfile"))
    .arg("--working-directory")
    .arg(&tmp.path().join("sub"))
    .output()?;

  if !output.status.success() {
    eprintln!("{:?}", String::from_utf8_lossy(&output.stderr));
    panic!();
  }

  let stdout = String::from_utf8(output.stdout).unwrap();

  assert_eq!(stdout, WANT);

  Ok(())
}

/// Test that just runs with the correct working directory when invoked with
/// `--justfile` but not `--working-directory`
#[test]
fn search_dir_child() -> Result<(), Box<dyn Error>> {
  let tmp = temptree! {
    child: {
      justfile: JUSTFILE,
      data: DATA,
    },
  };

  let output = Command::new(executable_path("just"))
    .current_dir(&tmp.path())
    .arg("child/")
    .output()?;

  if !output.status.success() {
    eprintln!("{:?}", String::from_utf8_lossy(&output.stderr));
    panic!();
  }

  let stdout = String::from_utf8(output.stdout).unwrap();

  assert_eq!(stdout, WANT);

  Ok(())
}

/// Test that just runs with the correct working directory when invoked with
/// `--justfile` but not `--working-directory`
#[test]
fn search_dir_parent() -> Result<(), Box<dyn Error>> {
  let tmp = temptree! {
    child: {
    },
    justfile: JUSTFILE,
    data: DATA,
  };

  let output = Command::new(executable_path("just"))
    .current_dir(&tmp.path().join("child"))
    .arg("../")
    .output()?;

  if !output.status.success() {
    eprintln!("{:?}", String::from_utf8_lossy(&output.stderr));
    panic!();
  }

  let stdout = String::from_utf8(output.stdout).unwrap();

  assert_eq!(stdout, WANT);

  Ok(())
}
