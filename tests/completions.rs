use std::process::Command;

use executable_path::executable_path;

#[test]
fn output() {
  let output = Command::new(executable_path("just"))
    .arg("--completions")
    .arg("bash")
    .output()
    .unwrap();

  assert!(output.status.success());

  let text = String::from_utf8_lossy(&output.stdout);

  assert!(text.starts_with("_just() {"));
}
