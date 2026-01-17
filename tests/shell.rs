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
    .success();
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
    .success();
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
    .success();
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
    .success();
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
    .failure();
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
    .failure();
}
