use super::*;

const JUSTFILE: &str = "
expression := `EXPRESSION`

recipe default=`DEFAULT`:
  {{expression}}
  {{default}}
  RECIPE
";

/// Test that --shell correctly sets the shell
#[test]
#[cfg_attr(windows, ignore)]
fn flag() {
  let tmp = temptree! {
    justfile: JUSTFILE,
    shell: "#!/usr/bin/env bash\necho \"$@\"",
  };

  let shell = tmp.path().join("shell");

  #[cfg(not(windows))]
  {
    let permissions = std::os::unix::fs::PermissionsExt::from_mode(0o700);
    fs::set_permissions(&shell, permissions).unwrap();
  }

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--shell")
    .arg(shell)
    .output()
    .unwrap();

  let stdout = "-cu -cu EXPRESSION\n-cu -cu DEFAULT\n-cu RECIPE\n";
  assert_stdout(&output, stdout);
}

/// Test that we can use `set shell` to use cmd.exe on windows
#[test]
#[cfg(windows)]
fn cmd() {
  let tmp = temptree! {
    justfile: r#"

set shell := ["cmd.exe", "/C"]

x := `Echo`

recipe:
  REM foo
  Echo "{{x}}"
"#,
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .output()
    .unwrap();

  let stdout = "\\\"ECHO is on.\\\"\r\n";

  assert_stdout(&output, stdout);
}

/// Test that we can use `set shell` to use cmd.exe on windows
#[test]
#[cfg(windows)]
fn powershell() {
  let tmp = temptree! {
      justfile: r#"

set shell := ["powershell.exe", "-c"]

x := `Write-Host "Hello, world!"`

recipe:
  For ($i=0; $i -le 10; $i++) { Write-Host $i }
  Write-Host "{{x}}"
"#
  ,
    };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .output()
    .unwrap();

  let stdout = "0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\nHello, world!\n";

  assert_stdout(&output, stdout);
}

#[test]
fn shell_args() {
  Test::new()
    .arg("--shell-arg")
    .arg("-c")
    .justfile(
      "
    default:
      echo A${foo}A
  ",
    )
    .shell(false)
    .stdout("AA\n")
    .stderr("echo A${foo}A\n")
    .run();
}

#[test]
fn shell_override() {
  Test::new()
    .arg("--shell")
    .arg("bash")
    .justfile(
      "
    set shell := ['foo-bar-baz']

    default:
      echo hello
  ",
    )
    .shell(false)
    .stdout("hello\n")
    .stderr("echo hello\n")
    .run();
}

#[test]
fn shell_arg_override() {
  Test::new()
    .arg("--shell-arg")
    .arg("-cu")
    .justfile(
      "
    set shell := ['foo-bar-baz']

    default:
      echo hello
  ",
    )
    .stdout("hello\n")
    .stderr("echo hello\n")
    .shell(false)
    .run();
}

#[test]
fn set_shell() {
  Test::new()
    .justfile(
      "
    set shell := ['echo', '-n']

    x := `bar`

    foo:
      echo {{x}}
      echo foo
  ",
    )
    .stdout("echo barecho foo")
    .stderr("echo bar\necho foo\n")
    .shell(false)
    .run();
}

#[test]
fn recipe_shell_not_found_error_message() {
  Test::new()
    .justfile(
      "
        foo:
          @echo bar
      ",
    )
    .shell(false)
    .args(["--shell", "NOT_A_REAL_SHELL"])
    .stderr_regex(
      "error: Recipe `foo` could not be run because just could not find the shell: .*\n",
    )
    .status(1)
    .run();
}

#[test]
fn backtick_recipe_shell_not_found_error_message() {
  Test::new()
    .justfile(
      "
        bar := `echo bar`

        foo:
          echo {{bar}}
      ",
    )
    .shell(false)
    .args(["--shell", "NOT_A_REAL_SHELL"])
    .stderr_regex("(?s)error: Backtick could not be run because just could not find the shell:.*")
    .status(1)
    .run();
}

/// Test that shell resolution respects PATH order
/// This verifies that executables are resolved according to PATH order,
/// not system-specific priority (like System32 on Windows)
#[test]
fn shell_respects_path_order() {
  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  // Create a custom shell in a directory that comes before system PATH
  let custom_dir = path.join("custom");
  fs::create_dir_all(&custom_dir).unwrap();

  // Create a wrapper script that uses the real sh but identifies itself
  // We'll use a simple approach: create a script that calls sh
  let script = if cfg!(windows) {
    r#"@echo off
echo CUSTOM_SHELL
sh -c %*
"#
  } else {
    "#!/bin/sh\necho CUSTOM_SHELL\nsh -c \"$@\"\n"
  };

  Test::with_tempdir(tmp)
    .write("custom/sh.exe", script)
    .make_executable("custom/sh.exe")
    .justfile(
      "
        set shell := ['sh.exe', '-c']

        default:
          echo hello
      ",
    )
    .env("PATH", custom_dir.to_str().unwrap())
    .shell(false)
    // Should use our custom sh.exe wrapper
    .stdout(if cfg!(windows) {
      "CUSTOM_SHELL\r\nhello\r\n"
    } else {
      "CUSTOM_SHELL\nhello\n"
    })
    .run();
}

/// Test that executable not found gives a clear error message
#[test]
fn shell_executable_not_found_error() {
  Test::new()
    .justfile(
      "
        set shell := ['nonexistent-shell-xyz123', '-c']

        default:
          echo hello
      ",
    )
    .shell(false)
    .stderr_regex(r"(?s).*error: Could not find executable `nonexistent-shell-xyz123` in PATH.*")
    .status(EXIT_FAILURE)
    .run();
}

/// Test that absolute paths work for shell setting
#[test]
fn shell_absolute_path() {
  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());
  let shell_path = path.join("custom-shell.exe");

  let script = if cfg!(windows) {
    r#"@echo off
echo custom
%*
"#
  } else {
    "#!/bin/sh\necho custom\nexec sh -c \"$@\"\n"
  };

  Test::with_tempdir(tmp)
    .write("custom-shell.exe", script)
    .make_executable("custom-shell.exe")
    .justfile(format!(
      "
        set shell := ['{}', '-c']

        default:
          echo hello
      ",
      shell_path.display()
    ))
    .shell(false)
    .stdout(if cfg!(windows) {
      "custom\r\nhello\r\n"
    } else {
      "custom\nhello\n"
    })
    .run();
}
