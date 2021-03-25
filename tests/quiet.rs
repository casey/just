use crate::common::*;

test! {
  name:     no_stdout,
  justfile: r#"
default:
  @echo hello
"#,
  args:     ("--quiet"),
  stdout:   "",
}

test! {
  name:     stderr,
  justfile: r#"
default:
  @echo hello 1>&2
"#,
  args:     ("--quiet"),
  stdout:   "",
}

test! {
  name:     command_echoing,
  justfile: r#"
default:
  exit
"#,
  args:     ("--quiet"),
  stdout:   "",
}

test! {
  name:     error_messages,
  justfile: r#"
default:
  exit 100
"#,
  args:     ("--quiet"),
  stdout:   "",
  status:   100,
}

test! {
  name:     assignment_backtick_stderr,
  justfile: r#"
a := `echo hello 1>&2`
default:
  exit 100
"#,
  args:     ("--quiet"),
  stdout:   "",
  status:   100,
}

test! {
  name:     interpolation_backtick_stderr,
  justfile: r#"
default:
  echo `echo hello 1>&2`
  exit 100
"#,
  args:     ("--quiet"),
  stdout:   "",
  status:   100,
}

test! {
  name: warning,
  justfile: "
    foo = 'bar'

    baz:
  ",
  args: ("--quiet"),
}

test! {
  name: choose_none,
  justfile: "",
  args: ("--choose", "--quiet"),
  status: EXIT_FAILURE,
}

test! {
  name: choose_invocation,
  justfile: "foo:",
  args: ("--choose", "--quiet", "--shell", "asdfasdfasfdasdfasdfadsf"),
  status: EXIT_FAILURE,
  shell: false,
}

test! {
  name: choose_status,
  justfile: "foo:",
  args: ("--choose", "--quiet", "--chooser", "/usr/bin/env false"),
  status: EXIT_FAILURE,
}

test! {
  name: edit_invocation,
  justfile: "foo:",
  args: ("--edit", "--quiet"),
  env: {
    "VISUAL": "adsfasdfasdfadsfadfsaf",
  },
  status: EXIT_FAILURE,
}

test! {
  name: edit_status,
  justfile: "foo:",
  args: ("--edit", "--quiet"),
  env: {
    "VISUAL": "false",
  },
  status: EXIT_FAILURE,
}

test! {
  name: init_exists,
  justfile: "foo:",
  args: ("--init", "--quiet"),
  status: EXIT_FAILURE,
}

test! {
  name: show_missing,
  justfile: "foo:",
  args: ("--show", "bar", "--quiet"),
  status: EXIT_FAILURE,
}

test! {
  name: summary_none,
  justfile: "",
  args: ("--summary", "--quiet"),
}

test! {
  name: quiet_shebang,
  justfile: "
    @foo:
      #!/bin/sh
  ",
  args: ("--quiet"),
}
