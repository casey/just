use super::*;

#[cfg(unix)]
use {
  nix::{sys::signal::Signal, unistd::Pid},
  std::process::{Child, Stdio},
  std::time::Duration,
};

#[cfg(unix)]
fn kill(child: &Child, signal: Signal) {
  nix::sys::signal::kill(Pid::from_raw(child.id().try_into().unwrap()), signal).unwrap();
}

#[test]
fn recipe_exit_message_suppressed() {
  Test::new()
    .justfile(
      "
      # This is a doc comment
      [no-exit-message]
      hello:
        @echo 'Hello, World!'
        @exit 100
      ",
    )
    .stdout("Hello, World!\n")
    .status(100);
}

#[test]
fn silent_recipe_exit_message_suppressed() {
  Test::new()
    .justfile(
      "
      # This is a doc comment
      [no-exit-message]
      @hello:
        echo 'Hello, World!'
        exit 100
      ",
    )
    .stdout("Hello, World!\n")
    .status(100);
}

#[test]
fn recipe_has_doc_comment() {
  Test::new()
    .justfile(
      "
    # This is a doc comment
    [no-exit-message]
    hello:
      @exit 100
        ",
    )
    .arg("--list")
    .stdout(
      "
      Available recipes:
          hello # This is a doc comment
      ",
    )
    .success();
}

#[test]
fn unknown_attribute() {
  Test::new()
    .justfile(
      "
      # This is a doc comment
      [unknown-attribute]
      hello:
        @exit 100
    ",
    )
    .stderr(
      "
      error: Unknown attribute `unknown-attribute`
       ——▶ justfile:2:2
        │
      2 │ [unknown-attribute]
        │  ^^^^^^^^^^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn empty_attribute() {
  Test::new()
    .justfile(
      "
      # This is a doc comment
      []
      hello:
        @exit 100
    ",
    )
    .stderr(
      "
      error: Expected identifier, but found ']'
       ——▶ justfile:2:2
        │
      2 │ []
        │  ^
      ",
    )
    .failure();
}

#[test]
fn extraneous_attribute_before_comment() {
  Test::new()
    .justfile(
      "
      [no-exit-message]
      # This is a doc comment
      hello:
        @exit 100
    ",
    )
    .stderr(
      "
      error: Extraneous attribute
       ——▶ justfile:1:1
        │
      1 │ [no-exit-message]
        │ ^
      ",
    )
    .failure();
}

#[test]
fn extraneous_attribute_before_empty_line() {
  Test::new()
    .justfile(
      "
      [no-exit-message]

      hello:
        @exit 100
    ",
    )
    .stderr(
      "
      error: Extraneous attribute
       ——▶ justfile:1:1
        │
      1 │ [no-exit-message]
        │ ^
    ",
    )
    .failure();
}

#[test]
fn shebang_exit_message_suppressed() {
  Test::new()
    .justfile(
      "
      [no-exit-message]
      hello:
        #!/usr/bin/env bash
        echo 'Hello, World!'
        exit 100
    ",
    )
    .stdout("Hello, World!\n")
    .status(100);
}

#[test]
fn no_exit_message() {
  Test::new()
    .justfile(
      "
      [no-exit-message]
      @hello:
        echo 'Hello, World!'
        exit 100
    ",
    )
    .stdout("Hello, World!\n")
    .status(100);
}

#[test]
fn exit_message() {
  Test::new()
    .justfile(
      "
      [exit-message]
      @hello:
        echo 'Hello, World!'
        exit 100
    ",
    )
    .stdout("Hello, World!\n")
    .stderr("error: Recipe `hello` failed on line 4 with exit code 100\n")
    .status(100);
}

#[test]
fn recipe_exit_message_setting_suppressed() {
  Test::new()
    .justfile(
      "
      set no-exit-message

      # This is a doc comment
      hello:
        @echo 'Hello, World!'
        @exit 100
    ",
    )
    .stdout("Hello, World!\n")
    .status(100);
}

#[test]
fn shebang_exit_message_setting_suppressed() {
  Test::new()
    .justfile(
      "
      set no-exit-message

      hello:
        #!/usr/bin/env bash
        echo 'Hello, World!'
        exit 100
    ",
    )
    .stdout("Hello, World!\n")
    .status(100);
}

#[test]
fn exit_message_override_no_exit_setting() {
  Test::new()
    .justfile(
      "
      set no-exit-message

      [exit-message]
      fail:
        @exit 100
    ",
    )
    .stderr("error: Recipe `fail` failed on line 5 with exit code 100\n")
    .status(100);
}

#[test]
fn exit_message_and_no_exit_message_compile_forbidden() {
  Test::new()
    .justfile(
      "
      [exit-message, no-exit-message]
      bar:
    ",
    )
    .stderr(
      "
        error: Recipe `bar` has both `[exit-message]` and `[no-exit-message]` attributes
         ——▶ justfile:2:1
          │
        2 │ bar:
          │ ^^^
      ",
    )
    .failure();
}

/// Verify that `[no-exit-message]` suppresses the error printed when a recipe
/// is terminated by a signal.
#[test]
#[ignore]
#[cfg(unix)]
fn signal_exit_message_suppressed() {
  let tmp = tempdir();

  fs::write(
    tmp.path().join("justfile"),
    unindent(
      "
        [no-exit-message]
        default:
          @sleep 1
      ",
    ),
  )
  .unwrap();

  let start = std::time::Instant::now();

  let mut child = Command::new(JUST)
    .current_dir(&tmp)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("just invocation failed");

  while start.elapsed() < Duration::from_millis(500) {}

  kill(&child, Signal::SIGINT);

  let output = child.wait_with_output().unwrap();
  let stderr = str::from_utf8(&output.stderr).unwrap();

  assert_eq!(
    stderr, "",
    "[no-exit-message] should suppress signal error, got: {stderr:?}"
  );
  assert_eq!(output.status.code(), Some(130));
}

/// Verify that without `[no-exit-message]` the signal error IS printed.
#[test]
#[ignore]
#[cfg(unix)]
fn signal_exit_message_not_suppressed() {
  let tmp = tempdir();

  fs::write(
    tmp.path().join("justfile"),
    unindent(
      "
        default:
          @sleep 1
      ",
    ),
  )
  .unwrap();

  let start = std::time::Instant::now();

  let mut child = Command::new(JUST)
    .current_dir(&tmp)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("just invocation failed");

  while start.elapsed() < Duration::from_millis(500) {}

  kill(&child, Signal::SIGINT);

  let output = child.wait_with_output().unwrap();
  let stderr = str::from_utf8(&output.stderr).unwrap();

  assert!(
    stderr.contains("was terminated"),
    "expected signal error message, got: {stderr:?}"
  );
  assert_eq!(output.status.code(), Some(130));
}

/// Verify that `set no-exit-message` also suppresses signal errors.
#[test]
#[ignore]
#[cfg(unix)]
fn signal_exit_message_setting_suppressed() {
  let tmp = tempdir();

  fs::write(
    tmp.path().join("justfile"),
    unindent(
      "
        set no-exit-message

        default:
          @sleep 1
      ",
    ),
  )
  .unwrap();

  let start = std::time::Instant::now();

  let mut child = Command::new(JUST)
    .current_dir(&tmp)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("just invocation failed");

  while start.elapsed() < Duration::from_millis(500) {}

  kill(&child, Signal::SIGINT);

  let output = child.wait_with_output().unwrap();
  let stderr = str::from_utf8(&output.stderr).unwrap();

  assert_eq!(
    stderr, "",
    "`set no-exit-message` should suppress signal error, got: {stderr:?}"
  );
  assert_eq!(output.status.code(), Some(130));
}
