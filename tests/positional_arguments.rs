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
  name: linewise_with_attribute,
  justfile: r#"
    [positional-arguments]
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
  name: shebang_with_attribute,
  justfile: "
    [positional-arguments]
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

test! {
  name: default_arguments,
  justfile: r#"
    set positional-arguments

    foo bar='baz':
      echo $1
  "#,
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
  justfile: r#"
    set positional-arguments

    foo *bar:
      echo $1
      echo $2
  "#,
  args:   ("foo", "a", "b"),
  stdout: "a\nb\n",
  stderr: "echo $1\necho $2\n",
}
