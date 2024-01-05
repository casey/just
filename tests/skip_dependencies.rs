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
    .args(&["--no-dep", "b"])
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
    .args(&["--no-dep", "b"])
    .stdout("b\n")
    .run();
}