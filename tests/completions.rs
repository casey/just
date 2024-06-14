use super::*;

#[test]
#[cfg(target_os = "linux")]
fn bash() {
  let output = Command::new(executable_path("just"))
    .args(["--completions", "bash"])
    .output()
    .unwrap();

  assert!(output.status.success());

  let script = str::from_utf8(&output.stdout).unwrap();

  let tempdir = tempdir();

  let path = tempdir.path().join("just.bash");

  fs::write(&path, script).unwrap();

  let status = Command::new("./tests/completions/just.bash")
    .arg(path)
    .status()
    .unwrap();

  assert!(status.success());
}

#[test]
fn replacements() {
  for shell in ["bash", "elvish", "fish", "nushell", "powershell", "zsh"] {
    let output = Command::new(executable_path("just"))
      .args(["--completions", shell])
      .output()
      .unwrap();
    assert!(
      output.status.success(),
      "shell completion generation for {shell} failed: {}",
      output.status
    );
  }
}
