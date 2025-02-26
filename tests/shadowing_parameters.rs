use super::*;

test! {
  name:     parameter_may_shadow_variable,
  justfile: "FOO := 'hello'\na FOO:\n echo {{FOO}}\n",
  args:     ("a", "bar"),
  stdout:   "bar\n",
  stderr:   "echo bar\n",
}

test! {
  name:     shadowing_parameters_do_not_change_environment,
  justfile: "export FOO := 'hello'\na FOO:\n echo $FOO\n",
  args:     ("a", "bar"),
  stdout:   "hello\n",
  stderr:   "echo $FOO\n",
}

test! {
  name:     exporting_shadowing_parameters_does_change_environment,
  justfile: "export FOO := 'hello'\na $FOO:\n echo $FOO\n",
  args:     ("a", "bar"),
  stdout:   "bar\n",
  stderr:   "echo $FOO\n",
}
