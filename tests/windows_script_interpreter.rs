use super::*;

#[test]
fn windows_script_interpreter_setting() {
  Test::new()
    .justfile(
      r#"
      set unstable
      set windows-script-interpreter := ["pwsh.exe", "-NoLogo", "-Command"]
      set script-interpreter := ["asdfasdfasdfasdf"]

      [script]
      foo:
        Write-Output bar
      "#,
    )
    .shell(false)
    .stdout("bar\r\n")
    .run();
}

#[test]
fn script_interpreter_setting_is_unstable() {
  Test::new()
    .justfile("set windows-script-interpreter := ['sh']")
    .status(EXIT_FAILURE)
    .stderr_regex(r"error: The `windows-script-interpreter` setting is currently unstable\..*")
    .run();
}

#[test]
fn overrides_script_interpreter() {
  Test::new()
    .justfile(
      r#"
      set unstable
      set script-interpreter := ["cmd.exe", "/c"]
      set windows-script-interpreter := ["pwsh.exe", "-NoLogo", "-Command"]

      [script]
      foo:
        Write-Output bar
      "#,
    )
    .shell(false)
    .stdout("bar\r\n")
    .run();
}
