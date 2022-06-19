use super::*;

#[test]
fn output() {
  let tempdir = tempdir();

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
