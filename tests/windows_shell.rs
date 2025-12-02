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
fn windows_powershell_setting_uses_powershell_set_shell() {
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

#[test]
fn windows_powershell_setting_uses_powershell() {
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

/// Test that Windows PATH resolution respects PATH order (not System32 priority)
/// This test verifies the fix for issue #2947
#[test]
#[cfg(windows)]
fn windows_path_resolution_respects_path_order() {
  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  // Create a custom shell in a test directory (simulating Git Bash)
  let custom_shell_dir = path.join("custom-shell");
  fs::create_dir_all(&custom_shell_dir).unwrap();

  // Create a simple shell script that echoes its path and executes the command
  let shell_script = r#"@echo off
echo CUSTOM_SHELL
%*
"#;

  Test::with_tempdir(tmp)
    .write("custom-shell/test-shell.exe", shell_script)
    .justfile(
      r#"
      set shell := ["test-shell.exe", "/c"]

      default:
        echo hello
      "#,
    )
    .env("PATH", custom_shell_dir.to_str().unwrap())
    .shell(false)
    // Should use our custom shell, not System32's
    .stdout("CUSTOM_SHELL\r\nhello\r\n")
    .run();
}

/// Test that PATHEXT extensions are checked on Windows
/// This test verifies the fix for issue #2926
#[test]
#[cfg(windows)]
fn windows_pathext_extensions_checked() {
  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  // Create a test executable without .exe extension
  let test_dir = path.join("test-dir");
  fs::create_dir_all(&test_dir).unwrap();

  let script = r#"@echo off
echo found
"#;

  Test::with_tempdir(tmp)
    .write("test-dir/testcmd.bat", script)
    .justfile(
      r#"
      set shell := ["testcmd", "/c"]

      default:
        echo hello
      "#,
    )
    .env("PATH", test_dir.to_str().unwrap())
    .shell(false)
    .stdout("found\r\nhello\r\n")
    .run();
}

/// Test that script-interpreter respects PATH order
#[test]
#[cfg(windows)]
fn windows_script_interpreter_path_resolution() {
  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  let custom_interpreter_dir = path.join("interpreter");
  fs::create_dir_all(&custom_interpreter_dir).unwrap();

  // Create an interpreter that echoes and then executes the script file
  let interpreter_script = r#"@echo off
echo CUSTOM_INTERPRETER
type %1
"#;

  Test::with_tempdir(tmp)
    .write("interpreter/custom.exe", interpreter_script)
    .justfile(
      r#"
      set unstable
      set script-interpreter := ["custom.exe", "/c"]

      [script]
      default:
        echo hello
      "#,
    )
    .env("PATH", custom_interpreter_dir.to_str().unwrap())
    .shell(false)
    .stdout("CUSTOM_INTERPRETER\r\necho hello\r\n")
    .run();
}

/// Test that shebang interpreters respect PATH order on Windows
#[test]
#[cfg(windows)]
fn windows_shebang_interpreter_path_resolution() {
  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  let custom_interpreter_dir = path.join("interpreter");
  fs::create_dir_all(&custom_interpreter_dir).unwrap();

  // Create an interpreter that echoes and then executes the script file
  let interpreter_script = r#"@echo off
echo SHEBANG_INTERPRETER
type %1
"#;

  Test::with_tempdir(tmp)
    .write("interpreter/python.exe", interpreter_script)
    .justfile(
      r#"
      default:
        #!python.exe
        echo hello
      "#,
    )
    .env("PATH", custom_interpreter_dir.to_str().unwrap())
    .shell(false)
    .stdout("SHEBANG_INTERPRETER\r\necho hello\r\n")
    .run();
}
