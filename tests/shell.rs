use super::*;

const JUSTFILE: &str = "
expression := `EXPRESSION`

recipe default=`DEFAULT`:
  {{expression}}
  {{default}}
  RECIPE
";

/// Test that --shell correctly sets the shell
#[cfg(not(windows))]
#[test]
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

const JUSTFILE_CMD: &str = r#"

set shell := ["cmd.exe", "/C"]

x := `Echo`

recipe:
  REM foo
  Echo "{{x}}"
"#;

/// Test that we can use `set shell` to use cmd.exe on windows
#[cfg(windows)]
#[test]
fn cmd() {
  let tmp = temptree! {
    justfile: JUSTFILE_CMD,
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .output()
    .unwrap();

  let stdout = "\\\"ECHO is on.\\\"\r\n";

  assert_stdout(&output, stdout);
}

const JUSTFILE_POWERSHELL: &str = r#"

set shell := ["powershell.exe", "-c"]

x := `Write-Host "Hello, world!"`

recipe:
  For ($i=0; $i -le 10; $i++) { Write-Host $i }
  Write-Host "{{x}}"
"#;

/// Test that we can use `set shell` to use cmd.exe on windows
#[cfg(windows)]
#[test]
fn powershell() {
  let tmp = temptree! {
    justfile: JUSTFILE_POWERSHELL,
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .output()
    .unwrap();

  let stdout = "0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\nHello, world!\n";

  assert_stdout(&output, stdout);
}

test! {
  name: shell_args,
  justfile: "
    default:
      echo A${foo}A
  ",
  args: ("--shell-arg", "-c"),
  stdout: "AA\n",
  stderr: "echo A${foo}A\n",
  shell: false,
}

test! {
  name: shell_override,
  justfile: "
    set shell := ['foo-bar-baz']

    default:
      echo hello
  ",
  args: ("--shell", "bash"),
  stdout: "hello\n",
  stderr: "echo hello\n",
  shell: false,
}

test! {
  name: shell_arg_override,
  justfile: "
    set shell := ['foo-bar-baz']

    default:
      echo hello
  ",
  args: ("--shell-arg", "-cu"),
  stdout: "hello\n",
  stderr: "echo hello\n",
  shell: false,
}

test! {
  name: set_shell,
  justfile: "
    set shell := ['echo', '-n']

    x := `bar`

    foo:
      echo {{x}}
      echo foo
  ",
  args: (),
  stdout: "echo barecho foo",
  stderr: "echo bar\necho foo\n",
  shell: false,
}
