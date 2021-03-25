test! {
  name: export_setting,
  justfile: "
    set export

    A := 'hello'

    foo B:
      echo $A
      echo $B
  ",
  args: ("foo", "goodbye"),
  stdout: "hello\ngoodbye\n",
  stderr: "echo $A\necho $B\n",
}

test! {
  name: export_shebang,
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
  name: export_override_undefined,
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
  name: export_variable_not_visible,
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
