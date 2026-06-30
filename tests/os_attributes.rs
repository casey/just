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

        [android]
        foo:
          echo babs
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
    } else if cfg!(target_os = "android") {
      "babs\n"
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
    } else if cfg!(target_os = "android") {
      "echo babs\n"
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
        [android]
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

#[test]
fn assignment() {
  Test::new()
    .justfile(
      "
        [unix]
        x := 'bar'

        [windows]
        x := 'baz'

        foo:
          @echo {{ x }}
      ",
    )
    .stdout(if cfg!(windows) { "baz\n" } else { "bar\n" })
    .success();
}

#[test]
fn alias() {
  Test::new()
    .justfile(
      "
        [unix]
        alias f := u

        [windows]
        alias f := w

        u:
          @echo unix

        w:
          @echo windows
      ",
    )
    .arg("f")
    .stdout(if cfg!(windows) { "windows\n" } else { "unix\n" })
    .success();
}

#[test]
fn setting() {
  Test::new()
    .justfile(
      "
        [unix]
        set quiet := true

        [windows]
        set quiet := false

        foo:
          echo bar
      ",
    )
    .stdout("bar\n")
    .stderr(if cfg!(windows) { "echo bar\n" } else { "" })
    .success();
}

#[test]
fn unexport() {
  Test::new()
    .justfile(
      "
        [unix]
        unexport JUST_TEST_VARIABLE

        @foo:
          echo ${JUST_TEST_VARIABLE:-unset}
      ",
    )
    .env("JUST_TEST_VARIABLE", "foo")
    .stdout(if cfg!(windows) { "foo\n" } else { "unset\n" })
    .success();
}

#[test]
fn function() {
  Test::new()
    .justfile(
      "
        [unix]
        f() := 'unix'

        [windows]
        f() := 'windows'

        foo:
          @echo {{ f() }}
      ",
    )
    .stdout(if cfg!(windows) { "windows\n" } else { "unix\n" })
    .unstable()
    .success();
}

#[test]
fn module() {
  Test::new()
    .justfile(
      "
        [unix]
        mod foo 'unix.just'

        [windows]
        mod foo 'windows.just'
      ",
    )
    .write("unix.just", "bar:\n  @echo unix\n")
    .write("windows.just", "bar:\n  @echo windows\n")
    .arg("foo::bar")
    .stdout(if cfg!(windows) { "windows\n" } else { "unix\n" })
    .success();
}

#[test]
fn disabled_module_file_not_required() {
  Test::new()
    .justfile(if cfg!(windows) {
      "
        [unix]
        mod foo

        bar:
          @echo bar
      "
    } else {
      "
        [windows]
        mod foo

        bar:
          @echo bar
      "
    })
    .arg("bar")
    .stdout("bar\n")
    .success();
}

#[test]
fn import() {
  Test::new()
    .justfile(
      "
        [unix]
        import 'unix.just'

        [windows]
        import 'windows.just'
      ",
    )
    .write("unix.just", "foo:\n  @echo unix\n")
    .write("windows.just", "foo:\n  @echo windows\n")
    .arg("foo")
    .stdout(if cfg!(windows) { "windows\n" } else { "unix\n" })
    .success();
}

#[test]
fn disabled_import_file_not_required() {
  Test::new()
    .justfile(if cfg!(windows) {
      "
        [unix]
        import 'foo.just'

        bar:
          @echo bar
      "
    } else {
      "
        [windows]
        import 'foo.just'

        bar:
          @echo bar
      "
    })
    .arg("bar")
    .stdout("bar\n")
    .success();
}
