use super::*;

#[cfg(windows)]
#[test]
fn powershell() {
  Test::new()
    .justfile(
      r#"
default:
  #!powershell
  Write-Host Hello-World
"#,
    )
    .stdout("Hello-World\n")
    .success();
}

#[cfg(windows)]
#[test]
fn powershell_exe() {
  Test::new()
    .justfile(
      r#"
default:
  #!powershell.exe
   Write-Host Hello-World
"#,
    )
    .stdout("Hello-World\n")
    .success();
}

#[cfg(windows)]
#[test]
fn cmd() {
  Test::new()
    .justfile(
      r#"
default:
  #!cmd /c
  @echo Hello-World
"#,
    )
    .stdout("Hello-World\r\n")
    .success();
}

#[cfg(windows)]
#[test]
fn cmd_exe() {
  Test::new()
    .justfile(
      r#"
default:
  #!cmd.exe /c
  @echo Hello-World
"#,
    )
    .stdout("Hello-World\r\n")
    .success();
}

#[cfg(windows)]
#[test]
fn multi_line_cmd_shebangs_are_removed() {
  Test::new()
    .justfile(
      r#"
default:
  #!cmd.exe /c
  #!foo
  @echo Hello-World
"#,
    )
    .stdout("Hello-World\r\n")
    .success();
}

#[test]
fn simple() {
  Test::new()
    .justfile(
      "
        foo:
          #!/bin/sh
          echo bar
      ",
    )
    .stdout("bar\n")
    .success();
}

#[test]
fn echo() {
  Test::new()
    .justfile(
      "
        @baz:
          #!/bin/sh
          echo fizz
      ",
    )
    .stdout("fizz\n")
    .stderr("#!/bin/sh\necho fizz\n")
    .success();
}

#[test]
fn echo_with_command_color() {
  Test::new()
    .justfile(
      "
        @baz:
          #!/bin/sh
          echo fizz
      ",
    )
    .args(["--color", "always", "--command-color", "purple"])
    .stdout("fizz\n")
    .stderr("\u{1b}[1;35m#!/bin/sh\u{1b}[0m\n\u{1b}[1;35mecho fizz\u{1b}[0m\n")
    .success();
}

// This test exists to make sure that shebang recipes run correctly.  Although
// this script is still executed by a shell its behavior depends on the value of
// a variable and continuing even though a command fails, whereas in plain
// recipes variables are not available in subsequent lines and execution stops
// when a line fails.
#[test]
fn run_shebang() {
  Test::new()
    .justfile(
      "
        a:
          #!/usr/bin/env sh
          code=200
          x() { return $code; }
          x
          x
      ",
    )
    .stderr("error: Recipe `a` failed with exit code 200\n")
    .status(200);
}
