use super::*;

#[test]
fn os_family() {
  Test::new()
    .justfile(
      "
      [unix]
      foo:
        echo bar

      [windows]
      foo:
        echo baz
    ",
    )
    .stdout(if cfg!(unix) {
      "bar\n"
    } else if cfg!(windows) {
      "baz\n"
    } else {
      panic!("unexpected os family")
    })
    .stderr(if cfg!(unix) {
      "echo bar\n"
    } else if cfg!(windows) {
      "echo baz\n"
    } else {
      panic!("unexpected os family")
    })
    .run();
}

#[test]
fn os() {
  Test::new()
    .justfile(
      "
      [macos]
      foo:
        echo bar

      [windows]
      foo:
        echo baz

      [linux]
      foo:
        echo quxx
    ",
    )
    .stdout(if cfg!(target_os = "macos") {
      "bar\n"
    } else if cfg!(windows) {
      "baz\n"
    } else if cfg!(linux) {
      "quxx\n"
    } else {
      panic!("unexpected os family")
    })
    .stderr(if cfg!(unix) {
      "echo bar\n"
    } else if cfg!(windows) {
      "echo baz\n"
    } else if cfg!(linux) {
      "echo quxx\n"
    } else {
      panic!("unexpected os family")
    })
    .run();
}
