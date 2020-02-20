use std::process::Command;

use executable_path::executable_path;
use tempfile::tempdir;

#[test]
fn output() {
  let tempdir = tempdir().unwrap();

  let output = Command::new(executable_path("just"))
    .arg("--completions")
    .arg("bash")
    .current_dir(tempdir.path())
    .output()
    .unwrap();

  assert!(output.status.success());

  let text = String::from_utf8_lossy(&output.stdout);

  assert!(text.starts_with("_just() {"));
}
