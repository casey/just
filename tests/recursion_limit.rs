use super::*;

#[test]
fn bugfix() {
  let mut justfile = String::from("foo: (x ");
  for _ in 0..500 {
    justfile.push('(');
  }
  Test::new()
    .justfile(&justfile)
    .stderr(RECURSION_LIMIT_REACHED)
    .status(EXIT_FAILURE)
    .run();
}

#[cfg(not(windows))]
const RECURSION_LIMIT_REACHED: &str = "
error: Parsing recursion depth exceeded
  |
1 | foo: (x ((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((
  |                                                                                                                                                                                                                                                                        ^
";

#[cfg(windows)]
const RECURSION_LIMIT_REACHED: &str = "
error: Parsing recursion depth exceeded
  |
1 | foo: (x ((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((
  |                                                                         ^
";
