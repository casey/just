use super::*;

#[test]
fn completion_scripts() {
  for shell in ["bash", "elvish", "fish", "nushell", "powershell", "zsh"] {
    Test::new()
      .args(["--completions", shell])
      .stdout_regex(if shell == "nushell" {
        ".*"
      } else {
        ".*JUST_COMPLETE.*"
      })
      .success();
  }
}
