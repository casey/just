use crate::common::*;

test! {
  name:     show,
  justfile: r#"hello := "foo"
bar := hello + hello
recipe:
 echo {{hello + "bar" + bar}}"#,
  args:     ("--show", "recipe"),
  stdout:   r#"
    recipe:
        echo {{ hello + "bar" + bar }}
  "#,
}

test! {
  name: alias_show,
  justfile: "foo:\n    bar\nalias f := foo",
  args: ("--show", "f"),
  stdout: "
    alias f := foo
    foo:
        bar
  ",
}

test! {
  name: alias_show_missing_target,
  justfile: "alias f := foo",
  args: ("--show", "f"),
  stderr: "
    error: Alias `f` has an unknown target `foo`
      |
    1 | alias f := foo
      |       ^
  ",
  status: EXIT_FAILURE,
}

test! {
  name:     show_suggestion,
  justfile: r#"
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

a Z="\t z":
"#,
  args:     ("--show", "hell"),
  stdout:   "",
  stderr:   "error: Justfile does not contain recipe `hell`.\nDid you mean `hello`?\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     show_alias_suggestion,
  justfile: r#"
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

alias foo := hello

a Z="\t z":
"#,
  args:     ("--show", "fo"),
  stdout:   "",
  stderr:   "
    error: Justfile does not contain recipe `fo`.
    Did you mean `foo`, an alias for `hello`?
  ",
  status:   EXIT_FAILURE,
}

test! {
  name:     show_no_suggestion,
  justfile: r#"
helloooooo a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

a Z="\t z":
"#,
  args:     ("--show", "hell"),
  stdout:   "",
  stderr:   "error: Justfile does not contain recipe `hell`.\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     show_no_alias_suggestion,
  justfile: r#"
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

alias foo := hello

a Z="\t z":
"#,
  args:     ("--show", "fooooooo"),
  stdout:   "",
  stderr:   "error: Justfile does not contain recipe `fooooooo`.\n",
  status:   EXIT_FAILURE,
}
