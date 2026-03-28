use super::*;

#[test]
fn trailing_newlines_are_stripped() {
  Test::new()
    .shell(false)
    .args(["--evaluate", "foos"])
    .justfile(
      "
set shell := ['python3', '-c']

foos := `print('foo' * 4)`
      ",
    )
    .stdout("foofoofoofoo")
    .success();
}

#[test]
fn backtick_with_powershell() {
  if !cfg!(windows) {
    return;
  }

  Test::new()
    .justfile(
      r#"
      set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

      foo := `Write-Output bar`

      default:
        @echo {{foo}}
    "#,
    )
    .shell(false)
    .stdout("bar\r\n")
    .success();
}
