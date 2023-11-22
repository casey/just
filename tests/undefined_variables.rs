use super::*;

#[test]
fn parameter_default_unknown_variable_in_expression() {
  Test::new()
    .justfile("foo a=(b+''):")
    .stderr(
      "
      error: Variable `b` not defined
       --> justfile:1:8
        |
      1 | foo a=(b+''):
        |        ^
    ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn unknown_variable_in_unary_call() {
  Test::new()
    .justfile(
      "
    foo x=env_var(a):
  ",
    )
    .stderr(
      "
      error: Variable `a` not defined
       --> justfile:1:15
        |
      1 | foo x=env_var(a):
        |               ^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn unknown_first_variable_in_binary_call() {
  Test::new()
    .justfile(
      "
    foo x=env_var_or_default(a, b):
  ",
    )
    .stderr(
      "
      error: Variable `a` not defined
       --> justfile:1:26
        |
      1 | foo x=env_var_or_default(a, b):
        |                          ^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn unknown_second_variable_in_binary_call() {
  Test::new()
    .justfile(
      "
    foo x=env_var_or_default('', b):
  ",
    )
    .stderr(
      "
      error: Variable `b` not defined
       --> justfile:1:30
        |
      1 | foo x=env_var_or_default('', b):
        |                              ^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn unknown_variable_in_ternary_call() {
  Test::new()
    .justfile(
      "
    foo x=replace(a, b, c):
  ",
    )
    .stderr(
      "
      error: Variable `a` not defined
       --> justfile:1:15
        |
      1 | foo x=replace(a, b, c):
        |               ^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}
