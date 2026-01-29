use super::*;

#[test]
fn then_branch_unevaluated() {
  Test::new()
    .justfile(
      "
    foo:
      echo {{ if 'a' == 'b' { `exit 1` } else { 'otherwise' } }}
  ",
    )
    .stdout("otherwise\n")
    .stderr("echo otherwise\n")
    .success();
}

#[test]
fn otherwise_branch_unevaluated() {
  Test::new()
    .justfile(
      "
    foo:
      echo {{ if 'a' == 'a' { 'then' } else { `exit 1` } }}
  ",
    )
    .stdout("then\n")
    .stderr("echo then\n")
    .success();
}

#[test]
fn otherwise_branch_unevaluated_inverted() {
  Test::new()
    .justfile(
      "
    foo:
      echo {{ if 'a' != 'b' { 'then' } else { `exit 1` } }}
  ",
    )
    .stdout("then\n")
    .stderr("echo then\n")
    .success();
}

#[test]
fn then_branch_unevaluated_inverted() {
  Test::new()
    .justfile(
      "
    foo:
      echo {{ if 'a' != 'a' { `exit 1` } else { 'otherwise' } }}
  ",
    )
    .stdout("otherwise\n")
    .stderr("echo otherwise\n")
    .success();
}

#[test]
fn complex_expressions() {
  Test::new()
    .justfile(
      "
    foo:
      echo {{ if 'a' + 'b' == `echo ab` { 'c' + 'd' } else { 'e' + 'f' } }}
  ",
    )
    .stdout("cd\n")
    .stderr("echo cd\n")
    .success();
}

#[test]
fn undefined_lhs() {
  Test::new()
    .justfile(
      "
    a := if b == '' { '' } else { '' }

    foo:
      echo {{ a }}
  ",
    )
    .stderr(
      "
    error: Variable `b` not defined
     ——▶ justfile:1:9
      │
    1 │ a := if b == '' { '' } else { '' }
      │         ^
  ",
    )
    .failure();
}

#[test]
fn undefined_rhs() {
  Test::new()
    .justfile(
      "
    a := if '' == b { '' } else { '' }

    foo:
      echo {{ a }}
  ",
    )
    .stderr(
      "
    error: Variable `b` not defined
     ——▶ justfile:1:15
      │
    1 │ a := if '' == b { '' } else { '' }
      │               ^
  ",
    )
    .failure();
}

#[test]
fn undefined_then() {
  Test::new()
    .justfile(
      "
    a := if '' == '' { b } else { '' }

    foo:
      echo {{ a }}
  ",
    )
    .stderr(
      "
    error: Variable `b` not defined
     ——▶ justfile:1:20
      │
    1 │ a := if '' == '' { b } else { '' }
      │                    ^
  ",
    )
    .failure();
}

#[test]
fn undefined_otherwise() {
  Test::new()
    .justfile(
      "
    a := if '' == '' { '' } else { b }

    foo:
      echo {{ a }}
  ",
    )
    .stderr(
      "
    error: Variable `b` not defined
     ——▶ justfile:1:32
      │
    1 │ a := if '' == '' { '' } else { b }
      │                                ^
  ",
    )
    .failure();
}

#[test]
fn unexpected_op() {
  Test::new()
    .justfile(
      "
    a := if '' a '' { '' } else { b }

    foo:
      echo {{ a }}
  ",
    )
    .stderr(
      "
    error: Expected '&&', '!=', '!~', '||', '==', '=~', '+', or '/', but found identifier
     ——▶ justfile:1:12
      │
    1 │ a := if '' a '' { '' } else { b }
      │            ^
  ",
    )
    .failure();
}

#[test]
fn dump() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    a := if '' == '' { '' } else { '' }

    foo:
      echo {{ a }}
  ",
    )
    .stdout(
      "
    a := if '' == '' { '' } else { '' }

    foo:
        echo {{ a }}
  ",
    )
    .success();
}

#[test]
fn if_else() {
  Test::new()
    .justfile(
      "
    x := if '0' == '1' { 'a' } else if '0' == '0' { 'b' } else { 'c' }

    foo:
      echo {{ x }}
  ",
    )
    .stdout("b\n")
    .stderr("echo b\n")
    .success();
}

#[test]
fn missing_else() {
  Test::new()
    .justfile(
      "
  TEST := if path_exists('/bin/bash') == 'true' {'yes'}
  ",
    )
    .stderr(
      "
    error: Expected keyword `else` but found `end of line`
     ——▶ justfile:1:54
      │
    1 │ TEST := if path_exists('/bin/bash') == 'true' {'yes'}
      │                                                      ^
  ",
    )
    .failure();
}

#[test]
fn incorrect_else_identifier() {
  Test::new()
    .justfile(
      "
  TEST := if path_exists('/bin/bash') == 'true' {'yes'} els {'no'}
  ",
    )
    .stderr(
      "
    error: Expected keyword `else` but found identifier `els`
     ——▶ justfile:1:55
      │
    1 │ TEST := if path_exists('/bin/bash') == 'true' {'yes'} els {'no'}
      │                                                       ^^^
  ",
    )
    .failure();
}
