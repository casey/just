use executable_path::executable_path;
use std::{error::Error, fs, process::Command};
use tempdir::TempDir;

/// Test that just runs with the correct working directory when invoked with
/// `--justfile` but not `--working-directory`
#[test]
fn justfile_without_working_directory() -> Result<(), Box<Error>> {
  let tmp = TempDir::new("just-integration")?;
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
