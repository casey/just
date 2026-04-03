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
    .success();
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

      [openbsd]
      foo:
        echo bob

      [freebsd]
      foo:
        echo corge

      [dragonfly]
      foo:
        echo grault

      [netbsd]
      foo:
        echo garply
    ",
    )
    .stdout(if cfg!(target_os = "macos") {
      "bar\n"
    } else if cfg!(windows) {
      "baz\n"
    } else if cfg!(target_os = "linux") {
      "quxx\n"
    } else if cfg!(target_os = "openbsd") {
      "bob\n"
    } else if cfg!(target_os = "freebsd") {
      "corge\n"
    } else if cfg!(target_os = "dragonfly") {
      "grault\n"
    } else if cfg!(target_os = "netbsd") {
      "garply\n"
    } else {
      panic!("unexpected os family")
    })
    .stderr(if cfg!(target_os = "macos") {
      "echo bar\n"
    } else if cfg!(windows) {
      "echo baz\n"
    } else if cfg!(target_os = "linux") {
      "echo quxx\n"
    } else if cfg!(target_os = "openbsd") {
      "echo bob\n"
    } else if cfg!(target_os = "freebsd") {
      "echo corge\n"
    } else if cfg!(target_os = "dragonfly") {
      "echo grault\n"
    } else if cfg!(target_os = "netbsd") {
      "echo garply\n"
    } else {
      panic!("unexpected os family")
    })
    .success();
}

#[test]
fn all() {
  Test::new()
    .justfile(
      "
      [linux]
      [macos]
      [openbsd]
      [freebsd]
      [dragonfly]
      [netbsd]
      [unix]
      [windows]
      foo:
        echo bar
    ",
    )
    .stdout("bar\n")
    .stderr("echo bar\n")
    .success();
}

#[test]
fn none() {
  Test::new()
    .justfile(
      "
      foo:
        echo bar
    ",
    )
    .stdout("bar\n")
    .stderr("echo bar\n")
    .success();
}
