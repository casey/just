use executable_path::executable_path;
use std::{process::Command, str};

fn stdout(reference: &str) -> String {
  let output = Command::new(executable_path("ref-type"))
    .args(["--reference", reference])
    .output()
    .unwrap();

  assert!(output.status.success());

  String::from_utf8(output.stdout).unwrap()
}

#[test]
fn junk_is_other() {
  assert_eq!(stdout("refs/tags/asdf"), "::set-output name=value::other\n");
}

#[test]
fn valid_version_is_release() {
  assert_eq!(
    stdout("refs/tags/0.0.0"),
    "::set-output name=value::release\n"
  );
}

#[test]
fn valid_version_with_trailing_characters_is_other() {
  assert_eq!(
    stdout("refs/tags/0.0.0-rc1"),
    "::set-output name=value::other\n"
  );
}

#[test]
fn valid_version_with_lots_of_digits_is_release() {
  assert_eq!(
    stdout("refs/tags/01232132.098327498374.43268473849734"),
    "::set-output name=value::release\n"
  );
}
