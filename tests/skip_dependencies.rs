use super::*;

#[test]
fn ignore_normal_dependency() {
  Test::new()
    .justfile(
      "
            a:
              echo 'A!'
            b: a
               echo 'B!'
            c:
               echo 'C!'
            ",
    )
    .args(&["--only", "b"])
    .stdout("B\n")
    .stderr("echo B\n")
    .run();
}

#[test]
fn ignore_prior_dependency() {
  Test::new()
    .justfile(
      "
            a:
              echo 'A!'
            b: a && c d
              echo 'B!'
            c:
              echo 'C!'
            d:
              echo 'D!'
            ",
    )
    .args(&["--only", "b"])
    .stdout("B\n")
    .stderr("echo B\n")
    .run();
}
