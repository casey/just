use super::*;

#[test]
fn test_one_of_parameter() {
  Test::new()
    .justfile(
      "
      enum foo := [ \"a\" = \"b\" ]

      test-enum p1=one-of(foo):
        @echo {{ p1 }}
      ",
    )
    .args(["test-enum", "a"])
    .stdout("b\n")
    .run();
}

#[test]
fn test_one_of_or_default_parameter() {
  Test::new()
    .justfile(
      "
      enum foo := [ \"a\" = \"b\" ]

      test-enum p1=one-of-or-default(foo, \"c\"):
        @echo {{ p1 }}
      ",
    )
    .args(["test-enum"])
    .stdout("c\n")
    .run();
}
