use super::*;

#[test]
fn ignore_normal_dependency() {
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
fn ignore_prior_dependency() {
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
fn ignore_dependency_multi() {
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
