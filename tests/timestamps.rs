use super::*;

#[test]
fn print_timestamps() {
  Test::new()
    .justfile(
      "
     recipe:
        echo 'one'
    ",
    )
    .arg("--timestamps")
    .stderr_regex(concat!(r"\[\d\d:\d\d:\d\d\] echo 'one'", "\n"))
    .stdout("one\n")
    .run();
}

#[test]
fn print_timestamps_with_format_string() {
  Test::new()
    .justfile(
      "
     recipe:
        echo 'one'
    ",
    )
    .args(["--timestamps", "--timestamp-format", "%H:%M:%S.%3f"])
    .stderr_regex(concat!(r"\[\d\d:\d\d:\d\d\.\d\d\d] echo 'one'", "\n"))
    .stdout("one\n")
    .run();
}
