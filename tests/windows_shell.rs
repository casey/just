use super::*;

#[test]
fn windows_shell_setting() {
  Test::new()
    .justfile(
      r#"
      set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

      foo:
        Write-Output bar
    "#,
    )
    .shell(false)
    .stdout("bar\r\n")
    .stderr("Write-Output bar\n")
    .run();
}
