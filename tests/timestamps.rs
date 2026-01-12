use super::*;

#[test]
fn print_timestamps_linewise() {
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
fn print_timestamps_script() {
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
fn print_timestamps_with_format_string_linewise() {
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
