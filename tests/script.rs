use super::*;

#[test]
fn unstable() {
  Test::new()
    .justfile(
      "
        [script('sh', '-u')]
        foo:
          echo FOO
      ",
    )
    .stderr_regex(r"error: The `\[script\]` attribute is currently unstable\..*")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn script_interpreter_setting_is_unstable() {
  Test::new()
    .justfile("set script-interpreter := ['sh']")
    .status(EXIT_FAILURE)
    .stderr_regex(r"error: The `script-interpreter` setting is currently unstable\..*")
    .run();
}

#[test]
fn runs_with_command() {
  Test::new()
    .justfile(
      "
        set unstable

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
    .run();
}

#[test]
fn no_arguments() {
  Test::new()
    .justfile(
      "
        set unstable

        [script('sh')]
        foo:
          echo foo
      ",
    )
    .stdout("foo\n")
    .run();
}

#[test]
fn with_arguments() {
  Test::new()
    .justfile(
      "
        set unstable

        [script('sh', '-x')]
        foo:
          echo foo
      ",
    )
    .stdout("foo\n")
    .stderr("+ echo foo\n")
    .run();
}

#[test]
fn not_allowed_with_shebang() {
  Test::new()
    .justfile(
      "
        set unstable

        [script('sh', '-u')]
        foo:
          #!/bin/sh

      ",
    )
    .stderr(
      "
        error: Recipe `foo` has both shebang line and `[script]` attribute
         ——▶ justfile:4:1
          │
        4 │ foo:
          │ ^^^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn script_line_numbers() {
  Test::new()
    .justfile(
      "
        set unstable

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
    .run();
}

#[test]
fn script_line_numbers_with_multi_line_recipe_signature() {
  Test::new()
    .justfile(
      r"
        set unstable

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
    .run();
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
    .run();
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
    .run();
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
    .run();
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
    .run();
}

#[test]
fn no_arguments_with_default_script_interpreter() {
  Test::new()
    .justfile(
      "
        set unstable

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
    .run();
}

#[test]
fn no_arguments_with_non_default_script_interpreter() {
  Test::new()
    .justfile(
      "
        set unstable

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
    .run();
}
