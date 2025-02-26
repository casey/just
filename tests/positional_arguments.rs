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

#[test]
fn variadic_shebang() {
  Test::new()
    .arg("foo")
    .arg("a")
    .arg("b")
    .arg("c")
    .justfile(r#"
    set positional-arguments

    foo *bar:
      #!/bin/sh
      echo $1
      echo "$@"
  "#)
    .stdout("a\na b c\n")
    .run();
}

#[test]
fn default_arguments() {
  Test::new()
    .justfile(r"
    set positional-arguments

    foo bar='baz':
      echo $1
  ")
    .stdout("baz\n")
    .stderr("echo $1\n")
    .run();
}

#[test]
fn empty_variadic_is_undefined() {
  Test::new()
    .justfile(r#"
    set positional-arguments

    foo *bar:
      if [ -n "${1+1}" ]; then echo defined; else echo undefined; fi
  "#)
    .stdout("undefined\n")
    .stderr("if [ -n \"${1+1}\" ]; then echo defined; else echo undefined; fi\n")
    .run();
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
