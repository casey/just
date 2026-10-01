use super::*;

#[test]
fn broken_pipe_is_ignored() {
  let tempdir = tempdir();

  fs::write(tempdir.path().join("justfile"), "foo:\n").unwrap();

  let (reader, writer) = io::pipe().unwrap();

  drop(reader);

  let output = Command::new(JUST)
    .arg("--list")
    .current_dir(tempdir.path())
    .stdin(Stdio::null())
    .stdout(writer)
    .stderr(Stdio::piped())
    .output()
    .unwrap();

  assert!(output.status.success());
  assert!(output.stderr.is_empty());
}
