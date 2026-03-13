use super::*;

#[test]
fn parameter_may_shadow_variable() {
  Test::new()
    .arg("a")
    .arg("bar")
    .justfile("FOO := 'hello'\na FOO:\n echo {{FOO}}\n")
    .stdout("bar\n")
    .stderr("echo bar\n")
    .success();
}

#[test]
fn shadowing_parameters_do_not_change_environment() {
  Test::new()
    .arg("a")
    .arg("bar")
    .justfile("export FOO := 'hello'\na FOO:\n echo $FOO\n")
    .stdout("hello\n")
    .stderr("echo $FOO\n")
    .success();
}

#[test]
fn exporting_shadowing_parameters_does_change_environment() {
  Test::new()
    .arg("a")
    .arg("bar")
    .justfile("export FOO := 'hello'\na $FOO:\n echo $FOO\n")
    .stdout("bar\n")
    .stderr("echo $FOO\n")
    .success();
}
