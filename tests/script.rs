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
          echo $UNSET_JUST_TEST_VARIABLE_ASDF
      ",
    )
    .stdout_regex(r"\n")
    .run();
}

#[test]
fn with_arguments() {
  Test::new()
    .justfile(
      "
        set unstable

        [script('sh', '-u')]
        foo:
          echo $UNSET_JUST_TEST_VARIABLE_ASDF
      ",
    )
    .stderr_regex(".*unbound variable.*")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn requires_argument() {
  Test::new()
    .justfile(
      "
        set unstable

        [script]
        foo:
      ",
    )
    .stderr(
      "
        error: Attribute `script` got 0 arguments but takes at least 1 argument
         ——▶ justfile:3:2
          │
        3 │ [script]
          │  ^^^^^^
      ",
    )
    .status(EXIT_FAILURE)
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
