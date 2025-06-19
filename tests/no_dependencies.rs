use super::*;

#[test]
fn skip_normal_dependency() {
  Test::new()
    .justfile(
      "
        a:
          @echo 'a'
        b: a
          @echo 'b'
        ",
    )
    .args(["--no-deps", "b"])
    .stdout("b\n")
    .run();
}

#[test]
fn skip_prior_dependency() {
  Test::new()
    .justfile(
      "
        a:
            @echo 'a'
        b: && a
            @echo 'b'
        ",
    )
    .args(["--no-deps", "b"])
    .stdout("b\n")
    .run();
}

#[test]
fn skip_recovery_deps() {
  Test::new()
    .justfile(
      "
          a: || b
              @echo 'a'
              exit 1
          b:
              @echo 'b'

          ",
    )
    .args(["--no-deps"])
    .stdout("a\n")
    .stderr(
      "
      exit 1
      error: Recipe `a` failed on line 3 with exit code 1
    ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn skip_dependency_multi() {
  Test::new()
    .justfile(
      "
          a:
              @echo 'a'
          b: && a
              @echo 'b'
          ",
    )
    .args(["--no-deps", "b", "a"])
    .stdout("b\na\n")
    .run();
}
