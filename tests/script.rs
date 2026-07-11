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

#[test]
fn shebang_line_numbers() {
  if cfg!(windows) {
    return;
  }
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

#[test]
fn shebang_line_numbers_with_multiline_constructs() {
  if cfg!(windows) {
    return;
  }
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

#[test]
fn multiline_shebang_line_numbers() {
  if cfg!(windows) {
    return;
  }
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

#[test]
fn shebang_line_numbers_windows() {
  if cfg!(not(windows)) {
    return;
  }
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
fn default_script_makes_recipes_scripts() {
  Test::new()
    .justfile(
      "
        set default-script

        foo:
          bar=baz
          echo $bar
      ",
    )
    .stdout("baz\n")
    .success();
}

#[test]
fn default_script_uses_script_interpreter() {
  Test::new()
    .justfile(
      "
        set default-script
        set script-interpreter := ['sh', '-x']

        foo:
          echo foo
      ",
    )
    .stdout("foo\n")
    .stderr("+ echo foo\n")
    .success();
}

#[test]
fn default_script_recipe_with_shebang_uses_shebang() {
  if cfg!(windows) {
    return;
  }
  Test::new()
    .justfile(
      "
        set default-script

        foo:
          #!/usr/bin/env cat
          bar
      ",
    )
    .stdout(
      "
        #!/usr/bin/env cat



        bar
      ",
    )
    .success();
}

#[test]
fn default_script_recipe_with_script_attribute() {
  Test::new()
    .justfile(
      "
        set default-script

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
fn default_script_false_is_shell() {
  Test::new()
    .justfile(
      "
        set default-script := false

        foo:
          echo foo
      ",
    )
    .stdout("foo\n")
    .stderr("echo foo\n")
    .success();
}

#[test]
fn default_script_allows_extra_leading_whitespace() {
  Test::new()
    .justfile(
      "
        set default-script

        foo:
          echo foo
            echo bar
      ",
    )
    .stdout("foo\nbar\n")
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

#[test]
fn shell_attribute_overrides_default_script() {
  Test::new()
    .justfile(
      "
        set default-script

        [shell]
        foo:
          echo foo
      ",
    )
    .stdout("foo\n")
    .stderr("echo foo\n")
    .success();
}

#[test]
fn shell_attribute_overrides_shebang() {
  Test::new()
    .justfile(
      "
        [shell]
        foo:
          #!/bin/sh
          echo foo
      ",
    )
    .stdout("foo\n")
    .stderr("#!/bin/sh\necho foo\n")
    .success();
}

#[test]
fn script_and_shell_attribute_forbidden() {
  Test::new()
    .justfile(
      "
        [script, shell]
        bar:
      ",
    )
    .stderr(
      "
        error: recipe `bar` has both `[script]` and `[shell]` attributes
         ——▶ justfile:2:1
          │
        2 │ bar:
          │ ^^^
      ",
    )
    .failure();
}

#[cfg(unix)]
#[test]
fn use_final_path_component_of_script_interpreter_to_determine_shell_kind() {
  Test::new()
    .write_executable(
      "pwsh.exe",
      "
        #!/bin/sh
        basename $1
      ",
    )
    .justfile(
      "
        [script('./pwsh.exe')]
        foo:
      ",
    )
    .arg("foo")
    .stdout("foo.ps1\n")
    .success();
}
