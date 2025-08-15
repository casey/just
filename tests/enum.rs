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
