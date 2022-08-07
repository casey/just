use super::*;

#[test]
fn windows_shell_setting() {
  Test::new()
    .justfile(
      r#"
      set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]
      set shell := ["asdfasdfasdfasdf"]

      foo:
        Write-Output bar
    "#,
    )
    .shell(false)
    .stdout("bar\r\n")
    .stderr("Write-Output bar\n")
    .run();
}

#[test]
fn windows_powershell_setting_uses_powershell() {
  Test::new()
    .justfile(
      r#"
      set windows-powershell
      set shell := ["asdfasdfasdfasdf"]

      foo:
        Write-Output bar
    "#,
    )
    .shell(false)
    .stdout("bar\r\n")
    .stderr("Write-Output bar\n")
    .run();
}
