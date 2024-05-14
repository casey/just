use super::*;

#[test]
fn trailing_newlines_are_stripped() {
  Test::new()
    .shell(false)
    .args(["--evaluate", "foos"])
    .justfile(
      "
set shell := ['python3', '-c']

foos := `print('foo' * 4)`
      ",
    )
    .stdout("foofoofoofoo")
    .run();
}
