use executable_path::executable_path;
use std::{process, str};

use test_utilities::tmptree;

#[test]
fn dotenv() {
  let tmp = tmptree! {
    ".env": "KEY=ROOT",
    sub: {
      ".env": "KEY=SUB",
      justfile: "default:\n\techo KEY=$KEY",
    },
  };

  let binary = executable_path("just");

  let output = process::Command::new(binary)
    .current_dir(tmp.path())
    .arg("sub/default")
    .output()
    .expect("just invocation failed");

  assert_eq!(output.status.code().unwrap(), 0);

  let stdout = str::from_utf8(&output.stdout).unwrap();
  assert_eq!(stdout, "KEY=SUB\n");
}
