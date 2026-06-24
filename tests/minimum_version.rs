use super::*;

#[test]
fn minimum_version_satisfied() {
  Test::new()
    .justfile(
      "
        set minimum-version := '0.0.0'

        foo:
          @echo bar
      ",
    )
    .stdout("bar\n")
    .success();
}

#[test]
fn minimum_version_too_low() {
  Test::new()
    .justfile(
      "
        set minimum-version := '999.0.0'

        foo:
          @echo bar
      ",
    )
    .stderr(format!(
      "
        error: justfile requires just 999.0.0 or later, but using {}
         ——▶ justfile:1:24
          │
        1 │ set minimum-version := '999.0.0'
          │                        ^^^^^^^^^
      ",
      env!("CARGO_PKG_VERSION"),
    ))
    .failure();
}

#[test]
fn minimum_version_invalid() {
  Test::new()
    .justfile("set minimum-version := 'foo'")
    .stderr(
      "
        error: setting `minimum-version` has invalid version `foo`: expected `MAJOR.MINOR.PATCH`
         ——▶ justfile:1:24
          │
        1 │ set minimum-version := 'foo'
          │                        ^^^^^
      ",
    )
    .failure();
}

#[test]
fn minimum_version_may_not_be_expression() {
  Test::new()
    .justfile("set minimum-version := ('1.' + '0')")
    .stderr(
      "
        error: `minimum-version` setting must be a plain string literal
         ——▶ justfile:1:5
          │
        1 │ set minimum-version := ('1.' + '0')
          │     ^^^^^^^^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn minimum_version_may_not_be_shell_expanded_string() {
  Test::new()
    .justfile("set minimum-version := x'1.0.0'")
    .stderr(
      "
        error: `minimum-version` setting must be a plain string literal
         ——▶ justfile:1:5
          │
        1 │ set minimum-version := x'1.0.0'
          │     ^^^^^^^^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn minimum_version_may_not_be_indented_string() {
  Test::new()
    .justfile("set minimum-version := '''1.0.0'''")
    .stderr(
      "
        error: `minimum-version` setting must be a plain string literal
         ——▶ justfile:1:5
          │
        1 │ set minimum-version := '''1.0.0'''
          │     ^^^^^^^^^^^^^^^
      ",
    )
    .failure();
}
