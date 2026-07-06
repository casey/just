use super::*;

#[test]
fn quiet() {
  Test::new()
    .justfile(
      "
        set quiet
        recipe:
          echo foo
      ",
    )
    .arg("--timestamp")
    .stderr_regex(concat!(r"\[\d\d:\d\d:\d\d\] echo foo", "\n"))
    .stdout("foo\n")
    .success();
}

#[test]
fn shell() {
  Test::new()
    .justfile(
      "
        recipe:
           echo 'one'
      ",
    )
    .arg("--timestamp")
    .stderr_regex(concat!(r"\[\d\d:\d\d:\d\d\] echo 'one'", "\n"))
    .stdout("one\n")
    .success();
}

#[test]
fn script() {
  Test::new()
    .justfile(
      "
        recipe:
           #!/bin/sh
           echo 'one'
      ",
    )
    .arg("--timestamp")
    .stderr_regex(concat!(r"\[\d\d:\d\d:\d\d\] recipe", "\n"))
    .stdout("one\n")
    .success();
}

#[test]
fn format_string() {
  Test::new()
    .justfile(
      "
        recipe:
           echo 'one'
      ",
    )
    .args(["--timestamp", "--timestamp-format", "%H:%M:%S.%3f"])
    .stderr_regex(concat!(r"\[\d\d:\d\d:\d\d\.\d\d\d] echo 'one'", "\n"))
    .stdout("one\n")
    .success();
}

#[test]
fn invalid_format_string_error() {
  Test::new()
    .justfile(
      "
        foo:
          @echo bar
      ",
    )
    .args(["--timestamp", "--timestamp-format", "%Q", "foo"])
    .stderr("error: failed to parse time format string `%Q`: bad or unsupported format string\n")
    .failure();
}
