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
    .failure();
}

#[test]
fn inline_module_recursion_limit() {
  let mut justfile = String::new();
  for i in 0..300 {
    justfile.push_str(&"  ".repeat(i));
    justfile.push_str("mod foo::\n");
  }
  Test::new()
    .justfile(justfile)
    .args(["--unstable", "--dump"])
    .stderr_regex("error: parsing recursion depth exceeded(.|\\n)*")
    .test_round_trip(false)
    .failure();
}

#[test]
fn user_defined_function_recursion_limit() {
  Test::new()
    .justfile(
      "
        set unstable

        foo() := foo()

        bar:
          echo {{foo()}}
      ",
    )
    .stderr(format!(
      "error: maximum recursion depth of {} exceeded while calling function foo\n",
      if cfg!(windows) { 48 } else { 256 },
    ))
    .failure();
}

const RECURSION_LIMIT_REACHED: &str = if cfg!(windows) {
  "
error: parsing recursion depth exceeded
 ——▶ justfile:1:57
  │
1 │ foo: (x ((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((
  │                                                         ^
"
} else {
  "
error: parsing recursion depth exceeded
 ——▶ justfile:1:265
  │
1 │ foo: (x ((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((
  │                                                                                                                                                                                                                                                                         ^
"
};
