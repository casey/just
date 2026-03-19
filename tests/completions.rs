use super::*;

#[test]
fn completions() {
  for shell in ["bash", "elvish", "fish", "nushell", "powershell", "zsh"] {
    let output = Command::new(JUST)
      .args(["--completions", shell])
      .output()
      .unwrap();
    assert!(
      output.status.success(),
      "shell completion generation for {shell} failed: {}\n{}",
      output.status,
      String::from_utf8_lossy(&output.stderr),
    );
    let script = String::from_utf8_lossy(&output.stdout);
    assert!(!script.is_empty(), "completion script for {shell} is empty");
  }
}
