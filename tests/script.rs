use super::*;

#[test]
fn runs_with_command() {
  Test::new()
    .justfile(
      "
        [script('cat')]
        foo:
          FOO
      ",
    )
    .stdout(
      "


        FOO
      ",
    )
    .success();
}

#[test]
fn no_arguments() {
  Test::new()
    .justfile(
      "
        [script('sh')]
        foo:
          echo foo
      ",
    )
    .stdout("foo\n")
    .success();
}

#[test]
fn with_arguments() {
  Test::new()
    .justfile(
      "
        [script('sh', '-x')]
        foo:
          echo foo
      ",
    )
    .stdout("foo\n")
    .stderr("+ echo foo\n")
    .success();
}

#[test]
fn allowed_with_shebang() {
  Test::new()
    .justfile(
      "
        [script('cat')]
        foo:
          #!/bin/sh
      ",
    )
    .stdout(
      "


        #!/bin/sh
      ",
    )
    .success();
}

#[test]
fn script_line_numbers() {
  Test::new()
    .justfile(
      "
        [script('cat')]
        foo:
          FOO

          BAR
      ",
    )
    .stdout(
      "


        FOO

        BAR
      ",
    )
    .success();
}

#[test]
fn script_line_numbers_with_multi_line_recipe_signature() {
  Test::new()
    .justfile(
      r"
        [script('cat')]
        foo bar='baz' \
          :
          FOO

          BAR

          {{ \
             bar \
          }}

          BAZ
      ",
    )
    .stdout(
      "



        FOO

        BAR

        baz



        BAZ
      ",
    )
    .success();
}

#[cfg(not(windows))]
#[test]
fn shebang_line_numbers() {
  Test::new()
    .justfile(
      "foo:
  #!/usr/bin/env cat

  a

  b


  c


",
    )
    .stdout(
      "#!/usr/bin/env cat


a

b


c
",
    )
    .success();
}

#[cfg(not(windows))]
#[test]
fn shebang_line_numbers_with_multiline_constructs() {
  Test::new()
    .justfile(
      r"foo b='b'\
        :
  #!/usr/bin/env cat

  a

  {{ \
     b \
  }}


  c


",
    )
    .stdout(
      "#!/usr/bin/env cat



a

b




c
",
    )
    .success();
}

#[cfg(not(windows))]
#[test]
fn multiline_shebang_line_numbers() {
  Test::new()
    .justfile(
      "foo:
  #!/usr/bin/env cat
  #!shebang
  #!shebang

  a

  b


  c


",
    )
    .stdout(
      "#!/usr/bin/env cat
#!shebang
#!shebang


a

b


c
",
    )
    .success();
}

#[cfg(windows)]
#[test]
fn shebang_line_numbers() {
  Test::new()
    .justfile(
      "foo:
  #!/usr/bin/env cat

  a

  b


  c


",
    )
    .stdout(
      "



a

b


c
",
    )
    .success();
}

#[test]
fn no_arguments_with_default_script_interpreter() {
  Test::new()
    .justfile(
      "
        [script]
        foo:
          case $- in
            *e*) echo '-e is set';;
          esac

          case $- in
            *u*) echo '-u is set';;
          esac
      ",
    )
    .stdout(
      "
        -e is set
        -u is set
      ",
    )
    .success();
}

#[test]
fn no_arguments_with_non_default_script_interpreter() {
  Test::new()
    .justfile(
      "
        set script-interpreter := ['sh']

        [script]
        foo:
          case $- in
            *e*) echo '-e is set';;
          esac

          case $- in
            *u*) echo '-u is set';;
          esac
      ",
    )
    .success();
}
