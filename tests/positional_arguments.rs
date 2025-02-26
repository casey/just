use super::*;

#[test]
fn linewise() {
  Test::new()
    .arg("foo")
    .arg("hello")
    .arg("goodbye")
    .justfile(r#"
    set positional-arguments

    foo bar baz:
      echo $0
      echo $1
      echo $2
      echo "$@"
  "#)
    .stdout("
    foo
    hello
    goodbye
    hello goodbye
  ")
    .stderr(r#"
    echo $0
    echo $1
    echo $2
    echo "$@"
  "#)
    .run();
}

#[test]
fn linewise_with_attribute() {
  Test::new()
    .arg("foo")
    .arg("hello")
    .arg("goodbye")
    .justfile(r#"
    [positional-arguments]
    foo bar baz:
      echo $0
      echo $1
      echo $2
      echo "$@"
  "#)
    .stdout("
    foo
    hello
    goodbye
    hello goodbye
  ")
    .stderr(r#"
    echo $0
    echo $1
    echo $2
    echo "$@"
  "#)
    .run();
}

#[test]
fn variadic_linewise() {
  Test::new()
    .args(&["foo", "a", "b", "c"])
    .justfile(r#"
    set positional-arguments

    foo *bar:
      echo $1
      echo "$@"
  "#)
    .stdout("a\na b c\n")
    .stderr("echo $1\necho \"$@\"\n")
    .run();
}

#[test]
fn shebang() {
  Test::new()
    .arg("foo")
    .arg("hello")
    .justfile("
    set positional-arguments

    foo bar:
      #!/bin/sh
      echo $1
  ")
    .stdout("hello\n")
    .run();
}

#[test]
fn shebang_with_attribute() {
  Test::new()
    .arg("foo")
    .arg("hello")
    .justfile("
    [positional-arguments]
    foo bar:
      #!/bin/sh
      echo $1
  ")
    .stdout("hello\n")
    .run();
}

test! {
  name: variadic_shebang,
  justfile: r#"
    set positional-arguments

    foo *bar:
      #!/bin/sh
      echo $1
      echo "$@"
  "#,
  args:   ("foo", "a", "b", "c"),
  stdout: "a\na b c\n",
}

test! {
  name: default_arguments,
  justfile: r"
    set positional-arguments

    foo bar='baz':
      echo $1
  ",
  args:   (),
  stdout: "baz\n",
  stderr: "echo $1\n",
}

test! {
  name: empty_variadic_is_undefined,
  justfile: r#"
    set positional-arguments

    foo *bar:
      if [ -n "${1+1}" ]; then echo defined; else echo undefined; fi
  "#,
  args:   (),
  stdout: "undefined\n",
  stderr: "if [ -n \"${1+1}\" ]; then echo defined; else echo undefined; fi\n",
}

test! {
  name: variadic_arguments_are_separate,
  justfile: r"
    set positional-arguments

    foo *bar:
      echo $1
      echo $2
  ",
  args:   ("foo", "a", "b"),
  stdout: "a\nb\n",
  stderr: "echo $1\necho $2\n",
}
