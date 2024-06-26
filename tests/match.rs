use super::*;

test! {
  name: other_branches_unevaluated,
  justfile: "
    foo:
      echo {{ match 'a' == 'b' { true => `exit 1`, false => 'otherwise' } }}
  ",
  stdout: "otherwise\n",
  stderr: "echo otherwise\n",
}

test! {
  name: otherwise_branch_unevaluated,
  justfile: "
    foo:
      echo {{ match 'a' == 'a' { true => 'then', _ => `exit 1` } }}
  ",
  stdout: "then\n",
  stderr: "echo then\n",
}

test! {
  name: complex_expressions,
  justfile: "
    foo:
      echo {{ match 'a' + 'b' == `echo ab` { true => 'c' + 'd', false => 'e' + 'f' }}
  ",
  stdout: "cd\n",
  stderr: "echo cd\n",
}

test! {
  name: undefined_expr,
  justfile: "
    a := match b == '' { _ => '' }

    foo:
      echo {{ a }}
  ",
  stdout: "",
  stderr: "
    error: Variable `b` not defined
     ——▶ justfile:1:11
      │
    1 │ a := match b == '' { _ => '' }
      │            ^
  ",
  status: EXIT_FAILURE,
}

test! {
  name: undefined_check,
  justfile: "
    a := match '' == '' { b => '' }

    foo:
      echo {{ a }}
  ",
  stdout: "",
  stderr: "
    error: Variable `b` not defined
     ——▶ justfile:1:23
      │
    1 │ a := match '' == '' { b => '' }
      │                       ^
  ",
  status: EXIT_FAILURE,
}

test! {
  name: undefined_branch,
  justfile: "
    a := match '' == '' { true => b }

    foo:
      echo {{ a }}
  ",
  stdout: "",
  stderr: "
    error: Variable `b` not defined
     ——▶ justfile:1:31
      │
    1 │ a := match '' == '' { true => b }
      │                               ^
  ",
  status: EXIT_FAILURE,
}

test! {
  name: undefined_otherwise,
  justfile: "
    a := match '' == '' { _ => b }

    foo:
      echo {{ a }}
  ",
  stdout: "",
  stderr: "
    error: Variable `b` not defined
     ——▶ justfile:1:28
      │
    1 │ a := match '' == '' { _ => b }
      │                            ^
  ",
  status: EXIT_FAILURE,
}

test! {
  name: dump,
  justfile: "
    a := match '' == '' { _ => '' }

    foo:
      echo {{ a }}
  ",
  args: ("--dump"),
  stdout: "
    a := match '' == '' { _ => '' }

    foo:
        echo {{ a }}
  ",
}

test! {
  name: if_else,
  justfile: "
    x := match '0' == '1' { true => 'a', false => 'b' }

    foo:
      echo {{ x }}
  ",
  stdout: "b\n",
  stderr: "echo b\n",
}

// TODO: test for failed match

// test! {
//   name: failed_match,
//   justfile: "
//   TEST := match '' == '' {}
//   ",
//   stdout: "",
//   stderr: "
//     error: Expected keyword `else` but found `end of line`
//      ——▶ justfile:1:54
//       │
//     1 │ TEST := if path_exists('/bin/bash') == 'true' {'yes'}
//       │                                                      ^
//   ",
//   status: EXIT_FAILURE,
// }
