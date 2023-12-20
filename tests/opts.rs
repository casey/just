use super::*;

// todo:
// - document
// - make unstable?
// - should opts be serialized?

#[test]
fn opts_are_dumped() {
  Test::new()
    .justfile("foo --bar BAR:")
    .arg("--dump")
    .stdout("foo --bar BAR:\n")
    .run();
}

#[test]
fn opts_may_take_values() {
  Test::new()
    .justfile(
      "
      foo --bar BAR:
        @echo {{ BAR }}
    ",
    )
    .args(["foo", "--bar", "baz"])
    .stdout("baz\n")
    .run();
}

#[test]
fn opts_may_be_parsed_after_arguments() {
  // foo BAR --bar BAZ:
  todo!()
}

#[test]
fn opts_may_be_passed_after_arguments() {
  // foo --bar BAZ BAR:
  // 'foo bar --bar baz
  todo!()
}

#[test]
fn opts_conflict_with_arguments() {
  // foo --bar BAR BAR:
  todo!()
}

#[test]
fn opts_conflict_with_each_other() {
  // foo --bar FOO --bar BAR:
  todo!()
}

#[test]
fn opts_can_be_passed_to_dependencies() {
  // foo: (bar --foo "hello")
  todo!()
}

#[test]
fn opts_can_be_passed_in_any_order() {
  // foo --bar BAR --baz BAZ:
  //
  // foo --baz a --bar b
  todo!()
}

#[test]
fn opts_without_default_values_are_mandatory() {
  // foo --bar BAR:
  //
  // foo
  todo!()
}

#[test]
fn opts_with_default_values_are_optional() {
  // foo --bar BAR="bar":
  //
  // foo
  todo!()
}

#[test]
fn opts_with_default_values_may_be_overridden() {
  // foo --bar BAR="bar":
  //
  // foo --bar "baz"
  todo!()
}

#[test]
fn dependency_invocations_with_same_opts_are_only_executed_once() {
  // foo --bar BAR:
  //
  // a: b c
  // b: (foo --bar "bar")
  // c: (foo --bar "bar")
  todo!()
}

#[test]
fn opts_with_no_value_are_an_error() {
  // foo --bar BAR:
  //
  //
  // foo --bar
  todo!()
}
