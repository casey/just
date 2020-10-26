test! {
  name: then_branch_unevaluated,
  justfile: "
    foo:
      echo {{ if 'a' == 'b' { `exit 1` } else { 'otherwise' } }}
  ",
  stdout: "otherwise\n",
  stderr: "echo otherwise\n",
}

test! {
  name: otherwise_branch_unevaluated,
  justfile: "
    foo:
      echo {{ if 'a' == 'a' { 'then' } else { `exit 1` } }}
  ",
  stdout: "then\n",
  stderr: "echo then\n",
}

test! {
  name: otherwise_branch_unevaluated_inverted,
  justfile: "
    foo:
      echo {{ if 'a' != 'b' { 'then' } else { `exit 1` } }}
  ",
  stdout: "then\n",
  stderr: "echo then\n",
}

test! {
  name: then_branch_unevaluated_inverted,
  justfile: "
    foo:
      echo {{ if 'a' != 'a' { `exit 1` } else { 'otherwise' } }}
  ",
  stdout: "otherwise\n",
  stderr: "echo otherwise\n",
}
