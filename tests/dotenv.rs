use super::*;

#[test]
fn dotenv() {
  Test::new()
    .write(".env", "KEY=ROOT")
    .write("sub/.env", "KEY=SUB")
    .write("sub/justfile", "default:\n\techo KEY=${KEY:-unset}")
    .args(["sub/default"])
    .stdout("KEY=unset\n")
    .stderr("echo KEY=${KEY:-unset}\n")
    .run();
}

test! {
  name:     set_false,
  justfile: r#"
    set dotenv-load := false

    foo:
      if [ -n "${DOTENV_KEY+1}" ]; then echo defined; else echo undefined; fi
  "#,
  stdout:   "undefined\n",
  stderr:   "if [ -n \"${DOTENV_KEY+1}\" ]; then echo defined; else echo undefined; fi\n",
}

test! {
  name:     set_implicit,
  justfile: r#"
    set dotenv-load

    foo:
      echo $DOTENV_KEY
  "#,
  stdout:   "dotenv-value\n",
  stderr:   "echo $DOTENV_KEY\n",
}

test! {
  name:     set_true,
  justfile: r#"
    set dotenv-load := true

    foo:
      echo $DOTENV_KEY
  "#,
  stdout:   "dotenv-value\n",
  stderr:   "echo $DOTENV_KEY\n",
}

#[test]
fn no_warning() {
  Test::new()
    .justfile(
      "
      foo:
        echo ${DOTENV_KEY:-unset}
    ",
    )
    .stdout("unset\n")
    .stderr("echo ${DOTENV_KEY:-unset}\n")
    .run();
}

#[test]
fn path_not_found() {
  Test::new()
    .justfile(
      "
      foo:
        echo $JUST_TEST_VARIABLE
    ",
    )
    .args(["--dotenv-path", ".env.prod"])
    .stderr(if cfg!(windows) {
      "error: Failed to load environment file: The system cannot find the file specified. (os \
       error 2)\n"
    } else {
      "error: Failed to load environment file: No such file or directory (os error 2)\n"
    })
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn path_resolves() {
  Test::new()
    .justfile(
      "
      foo:
        @echo $JUST_TEST_VARIABLE
    ",
    )
    .tree(tree! {
      subdir: {
        ".env": "JUST_TEST_VARIABLE=bar"
      }
    })
    .args(["--dotenv-path", "subdir/.env"])
    .stdout("bar\n")
    .status(EXIT_SUCCESS)
    .run();
}

#[test]
fn filename_resolves() {
  Test::new()
    .justfile(
      "
      foo:
        @echo $JUST_TEST_VARIABLE
    ",
    )
    .tree(tree! {
      ".env.special": "JUST_TEST_VARIABLE=bar"
    })
    .args(["--dotenv-filename", ".env.special"])
    .stdout("bar\n")
    .status(EXIT_SUCCESS)
    .run();
}

#[test]
fn filename_flag_overwrites_no_load() {
  Test::new()
    .justfile(
      "
      set dotenv-load := false

      foo:
        @echo $JUST_TEST_VARIABLE
    ",
    )
    .tree(tree! {
      ".env.special": "JUST_TEST_VARIABLE=bar"
    })
    .args(["--dotenv-filename", ".env.special"])
    .stdout("bar\n")
    .status(EXIT_SUCCESS)
    .run();
}

#[test]
fn path_flag_overwrites_no_load() {
  Test::new()
    .justfile(
      "
      set dotenv-load := false

      foo:
        @echo $JUST_TEST_VARIABLE
    ",
    )
    .tree(tree! {
      subdir: {
        ".env": "JUST_TEST_VARIABLE=bar"
      }
    })
    .args(["--dotenv-path", "subdir/.env"])
    .stdout("bar\n")
    .status(EXIT_SUCCESS)
    .run();
}

#[test]
fn can_set_dotenv_filename_from_justfile() {
  Test::new()
    .justfile(
      r#"
        set dotenv-filename := ".env.special"

        foo:
          @echo $JUST_TEST_VARIABLE
      "#,
    )
    .tree(tree! {
      ".env.special": "JUST_TEST_VARIABLE=bar"
    })
    .stdout("bar\n")
    .status(EXIT_SUCCESS)
    .run();
}

#[test]
fn can_set_dotenv_path_from_justfile() {
  Test::new()
    .justfile(
      r#"
        set dotenv-path := "subdir/.env"

        foo:
          @echo $JUST_TEST_VARIABLE
      "#,
    )
    .tree(tree! {
      subdir: {
        ".env": "JUST_TEST_VARIABLE=bar"
      }
    })
    .stdout("bar\n")
    .status(EXIT_SUCCESS)
    .run();
}

#[test]
fn program_argument_has_priority_for_dotenv_filename() {
  Test::new()
    .justfile(
      r#"
        set dotenv-filename := ".env.special"

        foo:
          @echo $JUST_TEST_VARIABLE
      "#,
    )
    .tree(tree! {
      ".env.special": "JUST_TEST_VARIABLE=bar",
      ".env.superspecial": "JUST_TEST_VARIABLE=baz"
    })
    .args(["--dotenv-filename", ".env.superspecial"])
    .stdout("baz\n")
    .status(EXIT_SUCCESS)
    .run();
}

#[test]
fn program_argument_has_priority_for_dotenv_path() {
  Test::new()
    .justfile(
      r#"
        set dotenv-path := "subdir/.env"

        foo:
          @echo $JUST_TEST_VARIABLE
      "#,
    )
    .tree(tree! {
      subdir: {
        ".env": "JUST_TEST_VARIABLE=bar",
        ".env.special": "JUST_TEST_VARIABLE=baz"
      }
    })
    .args(["--dotenv-path", "subdir/.env.special"])
    .stdout("baz\n")
    .status(EXIT_SUCCESS)
    .run();
}

#[test]
fn dotenv_path_is_relative_to_working_directory() {
  Test::new()
    .justfile(
      "
        set dotenv-path := '.env'

        foo:
          @echo $DOTENV_KEY
      ",
    )
    .tree(tree! { subdir: { } })
    .current_dir("subdir")
    .stdout("dotenv-value\n")
    .run();
}
