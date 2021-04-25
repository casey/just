test! {
  name: linewise,
  justfile: r#"
    set positional-arguments

    foo bar baz:
      echo $0
      echo $1
      echo $2
      echo "$@"
  "#,
  args:   ("foo", "hello", "goodbye"),
  stdout: "
    foo
    hello
    goodbye
    hello goodbye
  ",
  stderr: r#"
    echo $0
    echo $1
    echo $2
    echo "$@"
  "#,
}

test! {
  name: variadic_linewise,
  justfile: r#"
    set positional-arguments

    foo *bar:
      echo $1
      echo "$@"
  "#,
  args:   ("foo", "a", "b", "c"),
  stdout: "a\na b c\n",
  stderr: "echo $1\necho \"$@\"\n",
}

test! {
  name: shebang,
  justfile: "
    set positional-arguments

    foo bar:
      #!/bin/sh
      echo $1
  ",
  args:   ("foo", "hello"),
  stdout: "hello\n",
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
