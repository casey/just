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
        just --color <COLOR> --shell <SHELL> --shell-arg <SHELL-ARG>... \
        <--choose|--command <COMMAND>|--completions <SHELL>|--dump|--edit|\
        --evaluate|--init|--list|--show <RECIPE>|--summary|--variables>

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
  args: ("--command", "asdflkasdfjkasldkfjasldkfjasldkfjasdfkjasdf", "abc"),
  stderr: "
    error: Failed to invoke `asdflkasdfjkasldkfjasldkfjasldkfjasdfkjasdf` `abc`: \
    No such file or directory (os error 2)
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

test! {
  name: run_in_shell,
  justfile: "
    set shell := ['echo']
  ",
  args: ("--shell-command", "--command", "bar baz"),
  stdout: "bar baz\n",
  shell: false,
}

test! {
  name: exit_status,
  justfile: "
    x:
      echo XYZ
  ",
  args: ("--command", "false"),
  status: EXIT_FAILURE,
}

#[test]
fn working_directory_is_correct() {
  let tmp = tempdir();

  fs::write(tmp.path().join("justfile"), "").unwrap();
  fs::write(tmp.path().join("bar"), "baz").unwrap();
  fs::create_dir(tmp.path().join("foo")).unwrap();

  let output = Command::new(&executable_path("just"))
    .args(&["--command", "cat", "bar"])
    .current_dir(tmp.path().join("foo"))
    .output()
    .unwrap();

  assert_eq!(str::from_utf8(&output.stderr).unwrap(), "");

  assert!(output.status.success());

  assert_eq!(str::from_utf8(&output.stdout).unwrap(), "baz");
}
