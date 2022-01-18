use crate::common::*;

#[test]
fn windows_poweshell_setting_uses_powershell() {
  Test::new()
    .justfile(
      r#"
      set windows-powershell

      foo:
        Write-Output bar
    "#,
    )
    .shell(false)
    .stdout("bar\r\n")
    .stderr("Write-Output bar\n")
    .run();
}
