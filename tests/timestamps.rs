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
