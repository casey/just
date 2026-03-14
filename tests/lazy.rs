use super::*;

#[test]
fn lazy_is_unstable() {
  Test::new()
    .justfile(
      "
        set lazy

        foo:
      ",
    )
    .arg("foo")
    .stderr_regex(r"error: The `lazy` setting is currently unstable\. .*")
    .status(1);
}

#[test]
fn unused_assignment_not_evaluated() {
  Test::new()
    .justfile(
      "
      set lazy

      x := `exit 1`

      foo:
        @echo foo
    ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("foo\n")
    .success();
}

#[test]
fn used_assignment_evaluated() {
  Test::new()
    .justfile(
      "
      set lazy

      x := `exit 1`

      foo:
        @echo {{x}}
    ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: Backtick failed with exit code 1
         ——▶ justfile:3:6
          │
        3 │ x := `exit 1`
          │      ^^^^^^^^
      ",
    )
    .status(1);
}

#[test]
fn transitively_used_assignment_evaluated() {
  Test::new()
    .justfile(
      "
      set lazy

      x := `exit 1`
      y := x

      foo:
        @echo {{y}}
    ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: Backtick failed with exit code 1
         ——▶ justfile:3:6
          │
        3 │ x := `exit 1`
          │      ^^^^^^^^
      ",
    )
    .status(1);
}

#[test]
fn assignment_used_in_parameter_default_evaluated() {
  Test::new()
    .justfile(
      "
      set lazy

      x := `exit 1`

      foo val=x:
    ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: Backtick failed with exit code 1
         ——▶ justfile:3:6
          │
        3 │ x := `exit 1`
          │      ^^^^^^^^
      ",
    )
    .status(1);
}

#[test]
fn assignment_used_in_dependency_argument_evaluated() {
  Test::new()
    .justfile(
      "
      set lazy

      x := `exit 1`

      foo: (bar x)

      bar val:
    ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: Backtick failed with exit code 1
         ——▶ justfile:3:6
          │
        3 │ x := `exit 1`
          │      ^^^^^^^^
      ",
    )
    .status(1);
}

#[test]
fn assignment_in_body_interpolation_evaluated() {
  Test::new()
    .justfile(
      "
      set lazy

      x := `exit 1`

      foo:
        @echo {{x}}
    ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: Backtick failed with exit code 1
         ——▶ justfile:3:6
          │
        3 │ x := `exit 1`
          │      ^^^^^^^^
      ",
    )
    .status(1);
}

#[test]
fn multiple_invocations_evaluate_union() {
  Test::new()
    .justfile(
      "
      set lazy

      x := 'foo'
      y := 'bar'
      z := `exit 1`

      a:
        @echo {{x}}

      b:
        @echo {{y}}
    ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["a", "b"])
    .stdout("foo\nbar\n")
    .success();
}

#[test]
fn assignment_used_in_dependency_evaluated() {
  Test::new()
    .justfile(
      "
      set lazy

      x := `exit 1`

      foo: bar

      bar:
        @echo {{x}}
    ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: Backtick failed with exit code 1
         ——▶ justfile:3:6
          │
        3 │ x := `exit 1`
          │      ^^^^^^^^
      ",
    )
    .status(1);
}
