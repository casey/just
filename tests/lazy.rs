use super::*;

#[test]
fn lazy_variable_not_evaluated_if_unused() {
  Test::new()
    .justfile(
      "
        lazy expensive := `exit 1`

        works:
          @echo 'Success'
      ",
    )
    .stdout("Success\n")
    .run();
}

#[test]
fn lazy_variable_evaluated_when_used() {
  Test::new()
    .justfile(
      "
        lazy greeting := `echo 'Hello'`

        test:
          @echo {{greeting}}
      ",
    )
    .stdout("Hello\n")
    .run();
}

#[test]
fn lazy_variable_with_backtick_error() {
  Test::new()
    .justfile(
      "
        lazy bad := `exit 1`

        test:
          @echo {{bad}}
      ",
    )
    .stderr(
      "
        error: Backtick failed with exit code 1
         ——▶ justfile:1:13
          │
        1 │ lazy bad := `exit 1`
          │             ^^^^^^^^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn lazy_variable_used_multiple_times() {
  Test::new()
    .justfile(
      "
        lazy value := `echo 'test'`

        test:
          @echo {{value}}
          @echo {{value}}
      ",
    )
    .stdout("test\ntest\n")
    .run();
}

#[test]
fn lazy_and_export_are_separate() {
  Test::new()
    .justfile(
      "
        lazy foo := `echo 'lazy'`
        export bar := 'exported'

        test:
          @echo {{foo}} $bar
      ",
    )
    .stdout("lazy exported\n")
    .run();
}

#[test]
fn lazy_variable_dump() {
  Test::new()
    .justfile(
      "
        lazy greeting := `echo 'Hello'`
        normal := 'value'
      ",
    )
    .args(["--dump"])
    .stdout(
      "
        lazy greeting := `echo 'Hello'`
        normal := 'value'
      ",
    )
    .run();
}

#[test]
fn lazy_keyword_lexeme() {
  Test::new()
    .justfile(
      "
        lazy := 'not a keyword here'

        test:
          @echo {{lazy}}
      ",
    )
    .stdout("not a keyword here\n")
    .run();
}

#[test]
fn lazy_variable_in_dependency() {
  Test::new()
    .justfile(
      "
        lazy value := `echo 'computed'`

        dep:
          @echo {{value}}

        main: dep
          @echo 'done'
      ",
    )
    .args(["main"])
    .stdout("computed\ndone\n")
    .run();
}

#[test]
fn lazy_with_private() {
  Test::new()
    .justfile(
      "
        [private]
        lazy _secret := `echo 'hidden'`

        test:
          @echo {{_secret}}
      ",
    )
    .stdout("hidden\n")
    .run();
}

#[test]
fn lazy_variable_evaluated_once() {
  Test::new()
    .justfile(
      "
        lazy value := `date +%s%N`

        test:
          #!/usr/bin/env bash
          first={{value}}
          second={{value}}
          if [ \"$first\" = \"$second\" ]; then
            echo \"PASS: $first\"
          else
            echo \"FAIL: first=$first second=$second\"
            exit 1
          fi
      ",
    )
    .stdout_regex("^PASS: \\d+\\n$")
    .run();
}
