use super::*;

#[test]
fn success() {
  Test::new()
    .justfile(
      r#"
export FOO := "a"
baz := "c"
export BAR := "b"
export ABC := FOO + BAR + baz

wut:
  echo $FOO $BAR $ABC
"#,
    )
    .stdout("a b abc\n")
    .stderr("echo $FOO $BAR $ABC\n")
    .success();
}

#[test]
fn parameter() {
  Test::new()
    .justfile(
      r#"
    wut $FOO='a' BAR='b':
      echo $FOO
      echo {{BAR}}
      if [ -n "${BAR+1}" ]; then echo defined; else echo undefined; fi
  "#,
    )
    .stdout("a\nb\nundefined\n")
    .stderr(
      "echo $FOO\necho b\nif [ -n \"${BAR+1}\" ]; then echo defined; else echo undefined; fi\n",
    )
    .success();
}

#[test]
fn parameter_not_visible_to_backtick() {
  Test::new()
    .arg("wut")
    .arg("bar")
    .justfile(
      r#"
    wut $FOO BAR=`if [ -n "${FOO+1}" ]; then echo defined; else echo undefined; fi`:
      echo $FOO
      echo {{BAR}}
  "#,
    )
    .stdout("bar\nundefined\n")
    .stderr("echo $FOO\necho undefined\n")
    .success();
}

#[test]
fn override_variable() {
  Test::new()
    .arg("--set")
    .arg("BAR")
    .arg("bye")
    .arg("FOO=hello")
    .justfile(
      r#"
export FOO := "a"
baz := "c"
export BAR := "b"
export ABC := FOO + "-" + BAR + "-" + baz

wut:
  echo $FOO $BAR $ABC
"#,
    )
    .stdout("hello bye hello-bye-c\n")
    .stderr("echo $FOO $BAR $ABC\n")
    .success();
}

#[test]
fn shebang() {
  Test::new()
    .justfile(
      r#"
export FOO := "a"
baz := "c"
export BAR := "b"
export ABC := FOO + BAR + baz

wut:
  #!/bin/sh
  echo $FOO $BAR $ABC
"#,
    )
    .stdout("a b abc\n")
    .success();
}

#[test]
fn recipe_backtick() {
  Test::new()
    .justfile(
      r#"
export EXPORTED_VARIABLE := "A-IS-A"

recipe:
  echo {{`echo recipe $EXPORTED_VARIABLE`}}
"#,
    )
    .stdout("recipe A-IS-A\n")
    .stderr("echo recipe A-IS-A\n")
    .success();
}

#[test]
fn setting_implicit() {
  Test::new()
    .arg("foo")
    .arg("goodbye")
    .justfile(
      "
    set export

    A := 'hello'

    foo B C=`echo $A`:
      echo $A
      echo $B
      echo $C
  ",
    )
    .stdout("hello\ngoodbye\nhello\n")
    .stderr("echo $A\necho $B\necho $C\n")
    .success();
}

#[test]
fn setting_true() {
  Test::new()
    .justfile(
      "
    set export := true

    A := 'hello'

    foo B C=`echo $A`:
      echo $A
      echo $B
      echo $C
  ",
    )
    .arg("foo")
    .arg("goodbye")
    .stdout("hello\ngoodbye\nhello\n")
    .stderr("echo $A\necho $B\necho $C\n")
    .success();
}

#[test]
fn setting_false() {
  Test::new()
    .justfile(
      r#"
    set export := false

    A := 'hello'

    foo:
      if [ -n "${A+1}" ]; then echo defined; else echo undefined; fi
  "#,
    )
    .stdout("undefined\n")
    .stderr("if [ -n \"${A+1}\" ]; then echo defined; else echo undefined; fi\n")
    .success();
}

#[test]
fn setting_shebang() {
  Test::new()
    .arg("foo")
    .arg("goodbye")
    .justfile(
      "
    set export

    A := 'hello'

    foo B:
      #!/bin/sh
      echo $A
      echo $B
  ",
    )
    .stdout("hello\ngoodbye\n")
    .success();
}

#[test]
fn setting_override_undefined() {
  Test::new()
    .arg("A=zzz")
    .arg("foo")
    .justfile(
      r#"
    set export

    A := 'hello'
    B := `if [ -n "${A+1}" ]; then echo defined; else echo undefined; fi`

    foo C='goodbye' D=`if [ -n "${C+1}" ]; then echo defined; else echo undefined; fi`:
      echo $B
      echo $D
  "#,
    )
    .stdout("undefined\nundefined\n")
    .stderr("echo $B\necho $D\n")
    .success();
}

#[test]
fn setting_variable_not_visible() {
  Test::new()
    .arg("A=zzz")
    .justfile(
      r#"
    export A := 'hello'
    export B := `if [ -n "${A+1}" ]; then echo defined; else echo undefined; fi`

    foo:
      echo $B
  "#,
    )
    .stdout("undefined\n")
    .stderr("echo $B\n")
    .success();
}
