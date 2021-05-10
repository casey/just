use crate::common::*;

test! {
  name: long,
  justfile: "
    x:
      echo XYZ
  ",
  args: ("--command", "echo", "foo"),
  stdout: "foo\n",
}

test! {
  name: short,
  justfile: "
    x:
      echo XYZ
  ",
  args: ("-c", "echo", "foo"),
  stdout: "foo\n",
}

test! {
  name: no_binary,
  justfile: "
    x:
      echo XYZ
  ",
  args: ("--command"),
  stderr: "
    error: The argument '--command <COMMAND>' requires a value but none was supplied

    USAGE:
        just --color <COLOR> --shell <SHELL> --shell-arg <SHELL-ARG>... <--choose|--command <COMMAND>|--completions <SHELL>|--dump|--edit|--evaluate|--init|--list|--show <RECIPE>|--summary|--variables>

    For more information try --help
  ",
  status: EXIT_FAILURE,
}

test! {
  name: env_is_loaded,
  justfile: "
    x:
      echo XYZ
  ",
  args: ("--command", "sh", "-c", "echo $DOTENV_KEY"),
  stdout: "dotenv-value\n",
}

test! {
  name: exports_are_available,
  justfile: "
    export FOO := 'bar'

    x:
      echo XYZ
  ",
  args: ("--command", "sh", "-c", "echo $FOO"),
  stdout: "bar\n",
}

test! {
  name: command_not_found_error,
  justfile: "
    x:
      echo XYZ
  ",
  args: ("--command", "asdflkasdfjkasldkfjasldkfjasldkfjasdfkjasdf"),
  stderr: "
    SOME REASONABLE ERROR MESSAGE
  ",
  status: EXIT_FAILURE,
}

test! {
  name: set_overrides_work,
  justfile: "
    export FOO := 'bar'

    x:
      echo XYZ
  ",
  args: ("--set", "FOO", "baz", "--command", "sh", "-c", "echo $FOO"),
  stdout: "baz\n",
}

#[test]
fn working_directory_is_correct() {
  todo!()
}

#[test]
fn search_dir_allowed() {
  todo!()
  // `--command foo/ echo bar` runs inside foo
}
