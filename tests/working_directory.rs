mod tempdir;

use std::{error::Error, fs, process::Command};

use executable_path::executable_path;

use tempdir::tempdir;

/// Test that just runs with the correct working directory when invoked with
/// `--justfile` but not `--working-directory`
#[test]
fn justfile_without_working_directory() -> Result<(), Box<dyn Error>> {
  let tmp = tempdir();
  let justfile = tmp.path().join("justfile");
  let data = tmp.path().join("data");
  fs::write(
    &justfile,
    "foo = `cat data`\ndefault:\n echo {{foo}}\n cat data",
  )?;
  fs::write(&data, "found it")?;

  let output = Command::new(executable_path("just"))
    .arg("--justfile")
    .arg(&justfile)
    .output()?;

  if !output.status.success() {
    panic!()
  }

  let stdout = String::from_utf8(output.stdout).unwrap();

  assert_eq!(stdout, "found it\nfound it");

  Ok(())
}

/// Test that just invokes commands from the directory in which the justfile is found
#[test]
fn change_working_directory_to_justfile_parent() -> Result<(), Box<dyn Error>> {
  let tmp = tempdir();

  let justfile = tmp.path().join("justfile");
  fs::write(
    &justfile,
    "foo = `cat data`\ndefault:\n echo {{foo}}\n cat data",
  )?;

  let data = tmp.path().join("data");
  fs::write(&data, "found it")?;

  let subdir = tmp.path().join("subdir");
  fs::create_dir(&subdir)?;

  let output = Command::new(executable_path("just"))
    .current_dir(subdir)
    .output()?;

  if !output.status.success() {
    panic!("just invocation failed: {}", output.status)
  }

  let stdout = String::from_utf8(output.stdout).unwrap();

  assert_eq!(stdout, "found it\nfound it");

  Ok(())
}
