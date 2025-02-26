use super::*;

test! {
  name:     success,
  justfile: r#"
export FOO := "a"
baz := "c"
export BAR := "b"
export ABC := FOO + BAR + baz

wut:
  echo $FOO $BAR $ABC
"#,
  stdout:   "a b abc\n",
  stderr:   "echo $FOO $BAR $ABC\n",
}

test! {
  name:     parameter,
  justfile: r#"
    wut $FOO='a' BAR='b':
      echo $FOO
      echo {{BAR}}
      if [ -n "${BAR+1}" ]; then echo defined; else echo undefined; fi
  "#,
  stdout: "a\nb\nundefined\n",
  stderr: "echo $FOO\necho b\nif [ -n \"${BAR+1}\" ]; then echo defined; else echo undefined; fi\n",
}

test! {
  name:     parameter_not_visible_to_backtick,
  justfile: r#"
    wut $FOO BAR=`if [ -n "${FOO+1}" ]; then echo defined; else echo undefined; fi`:
      echo $FOO
      echo {{BAR}}
  "#,
  args:   ("wut", "bar"),
  stdout: "bar\nundefined\n",
  stderr: "echo $FOO\necho undefined\n",
}

test! {
  name:     override_variable,
  justfile: r#"
export FOO := "a"
baz := "c"
export BAR := "b"
export ABC := FOO + "-" + BAR + "-" + baz

wut:
  echo $FOO $BAR $ABC
"#,
  args:     ("--set", "BAR", "bye", "FOO=hello"),
  stdout:   "hello bye hello-bye-c\n",
  stderr:   "echo $FOO $BAR $ABC\n",
}

test! {
  name:     shebang,
  justfile: r#"
export FOO := "a"
baz := "c"
export BAR := "b"
export ABC := FOO + BAR + baz

wut:
  #!/bin/sh
  echo $FOO $BAR $ABC
"#,
  stdout:   "a b abc\n",
}

test! {
  name:     recipe_backtick,
  justfile: r#"
export EXPORTED_VARIABLE := "A-IS-A"

recipe:
  echo {{`echo recipe $EXPORTED_VARIABLE`}}
"#,
  stdout:   "recipe A-IS-A\n",
  stderr:   "echo recipe A-IS-A\n",
}

test! {
  name: setting_implicit,
  justfile: "
    set export

    A := 'hello'

    foo B C=`echo $A`:
      echo $A
      echo $B
      echo $C
  ",
  args: ("foo", "goodbye"),
  stdout: "hello\ngoodbye\nhello\n",
  stderr: "echo $A\necho $B\necho $C\n",
}

test! {
  name: setting_true,
  justfile: "
    set export := true

    A := 'hello'

    foo B C=`echo $A`:
      echo $A
      echo $B
      echo $C
  ",
  args: ("foo", "goodbye"),
  stdout: "hello\ngoodbye\nhello\n",
  stderr: "echo $A\necho $B\necho $C\n",
}

test! {
  name: setting_false,
  justfile: r#"
    set export := false

    A := 'hello'

    foo:
      if [ -n "${A+1}" ]; then echo defined; else echo undefined; fi
  "#,
  stdout: "undefined\n",
  stderr: "if [ -n \"${A+1}\" ]; then echo defined; else echo undefined; fi\n",
}

test! {
  name: setting_shebang,
  justfile: "
    set export

    A := 'hello'

    foo B:
      #!/bin/sh
      echo $A
      echo $B
  ",
  args: ("foo", "goodbye"),
  stdout: "hello\ngoodbye\n",
  stderr: "",
}

test! {
  name: setting_override_undefined,
  justfile: r#"
    set export

    A := 'hello'
    B := `if [ -n "${A+1}" ]; then echo defined; else echo undefined; fi`

    foo C='goodbye' D=`if [ -n "${C+1}" ]; then echo defined; else echo undefined; fi`:
      echo $B
      echo $D
  "#,
  args: ("A=zzz", "foo"),
  stdout: "undefined\nundefined\n",
  stderr: "echo $B\necho $D\n",
}

test! {
  name: setting_variable_not_visible,
  justfile: r#"
    export A := 'hello'
    export B := `if [ -n "${A+1}" ]; then echo defined; else echo undefined; fi`

    foo:
      echo $B
  "#,
  args: ("A=zzz"),
  stdout: "undefined\n",
  stderr: "echo $B\n",
}
