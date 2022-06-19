use super::*;

test! {
  name: mismatched_delimiter,
  justfile: "(]",
  stderr: "
    error: Mismatched closing delimiter `]`. (Did you mean to close the `(` on line 1?)
      |
    1 | (]
      |  ^
  ",
  status: EXIT_FAILURE,
}

test! {
  name: unexpected_delimiter,
  justfile: "]",
  stderr: "
    error: Unexpected closing delimiter `]`
      |
    1 | ]
      | ^
  ",
  status: EXIT_FAILURE,
}

test! {
  name: paren_continuation,
  justfile: "
    x := (
          'a'
              +
      'b'
    )

    foo:
      echo {{x}}
  ",
  stdout: "ab\n",
  stderr: "echo ab\n",
}

test! {
  name: brace_continuation,
  justfile: "
    x := if '' == '' {
      'a'
    } else {
      'b'
    }

    foo:
      echo {{x}}
  ",
  stdout: "a\n",
  stderr: "echo a\n",
}

test! {
  name: bracket_continuation,
  justfile: "
    set shell := [
      'sh',
      '-cu',
    ]

    foo:
      echo foo
  ",
  stdout: "foo\n",
  stderr: "echo foo\n",
}

test! {
  name: dependency_continuation,
  justfile: "
    foo: (
    bar 'bar'
    )
      echo foo

    bar x:
      echo {{x}}
  ",
  stdout: "bar\nfoo\n",
  stderr: "echo bar\necho foo\n",
}

test! {
  name: no_interpolation_continuation,
  justfile: "
    foo:
      echo {{ (
        'a' + 'b')}}
  ",
  stdout: "",
  stderr: "
    error: Unterminated interpolation
      |
    2 |   echo {{ (
      |        ^^
  ",
  status: EXIT_FAILURE,
}
