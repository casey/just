use super::*;

#[test]
fn linewise() {
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
    .run();
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
    .run();
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
    .run();
}
