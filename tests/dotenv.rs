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

#[test]
fn set_false() {
  Test::new()
    .justfile(
      r#"
      set dotenv-load := false

      @foo:
        if [ -n "${DOTENV_KEY+1}" ]; then echo defined; else echo undefined; fi
    "#,
    )
    .write(".env", "DOTENV_KEY=dotenv-value")
    .stdout("undefined\n")
    .run();
}

#[test]
fn set_implicit() {
  Test::new()
    .justfile(
      "
        set dotenv-load

        foo:
          echo $DOTENV_KEY
      ",
    )
    .write(".env", "DOTENV_KEY=dotenv-value")
    .stdout("dotenv-value\n")
    .stderr("echo $DOTENV_KEY\n")
    .run();
}

#[test]
fn set_true() {
  Test::new()
    .justfile(
      "
        set dotenv-load := true

        foo:
          echo $DOTENV_KEY
      ",
    )
    .write(".env", "DOTENV_KEY=dotenv-value")
    .stdout("dotenv-value\n")
    .stderr("echo $DOTENV_KEY\n")
    .run();
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
    .write(".env", "DOTENV_KEY=dotenv-value")
    .stdout("unset\n")
    .stderr("echo ${DOTENV_KEY:-unset}\n")
    .run();
}

#[test]
fn dotenv_required() {
  Test::new()
    .justfile(
      "
        set dotenv-required

        foo:
      ",
    )
    .stderr("error: Dotenv file not found\n")
    .status(1)
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
      "
        set dotenv-path := 'subdir/.env'

        foo:
          @echo $JUST_TEST_VARIABLE
      ",
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
    .write(".env", "DOTENV_KEY=dotenv-value")
    .tree(tree! { subdir: { } })
    .current_dir("subdir")
    .stdout("dotenv-value\n")
    .run();
}

#[test]
fn dotenv_variable_in_recipe() {
  Test::new()
    .justfile(
      "
        set dotenv-load

        echo:
          echo $DOTENV_KEY
      ",
    )
    .write(".env", "DOTENV_KEY=dotenv-value")
    .stdout("dotenv-value\n")
    .stderr("echo $DOTENV_KEY\n")
    .run();
}

#[test]
fn dotenv_variable_in_backtick() {
  Test::new()
    .justfile(
      "
        set dotenv-load
        X:=`echo $DOTENV_KEY`
        echo:
          echo {{X}}
      ",
    )
    .write(".env", "DOTENV_KEY=dotenv-value")
    .stdout("dotenv-value\n")
    .stderr("echo dotenv-value\n")
    .run();
}

#[test]
fn dotenv_variable_in_function_in_recipe() {
  Test::new()
    .justfile(
      "
        set dotenv-load
        echo:
          echo {{env_var_or_default('DOTENV_KEY', 'foo')}}
          echo {{env_var('DOTENV_KEY')}}
      ",
    )
    .write(".env", "DOTENV_KEY=dotenv-value")
    .stdout("dotenv-value\ndotenv-value\n")
    .stderr("echo dotenv-value\necho dotenv-value\n")
    .run();
}

#[test]
fn dotenv_variable_in_function_in_backtick() {
  Test::new()
    .justfile(
      "
  set dotenv-load
  X:=env_var_or_default('DOTENV_KEY', 'foo')
  Y:=env_var('DOTENV_KEY')
  echo:
    echo {{X}}
    echo {{Y}}
",
    )
    .write(".env", "DOTENV_KEY=dotenv-value")
    .stdout("dotenv-value\ndotenv-value\n")
    .stderr("echo dotenv-value\necho dotenv-value\n")
    .run();
}

#[test]
fn no_dotenv() {
  Test::new()
    .justfile(
      "
        X:=env_var_or_default('DOTENV_KEY', 'DEFAULT')
        echo:
          echo {{X}}
      ",
    )
    .write(".env", "DOTENV_KEY=dotenv-value")
    .arg("--no-dotenv")
    .stdout("DEFAULT\n")
    .stderr("echo DEFAULT\n")
    .run();
}

#[test]
fn dotenv_env_var_override() {
  Test::new()
    .justfile(
      "
        echo:
          echo $DOTENV_KEY
      ",
    )
    .write(".env", "DOTENV_KEY=dotenv-value")
    .env("DOTENV_KEY", "not-the-dotenv-value")
    .stdout("not-the-dotenv-value\n")
    .stderr("echo $DOTENV_KEY\n")
    .run();
}

#[test]
fn dotenv_path_usable_from_subdir() {
  Test::new()
    .justfile(
      "
        set dotenv-path := '.custom-env'

        @echo:
          echo $DOTENV_KEY
      ",
    )
    .create_dir("sub")
    .current_dir("sub")
    .write(".custom-env", "DOTENV_KEY=dotenv-value")
    .stdout("dotenv-value\n")
    .run();
}

#[test]
fn dotenv_path_does_not_override_dotenv_file() {
  Test::new()
    .write(".env", "KEY=ROOT")
    .write(
      "sub/justfile",
      "set dotenv-path := '.'\n@foo:\n echo ${KEY}",
    )
    .current_dir("sub")
    .stdout("ROOT\n")
    .run();
}

#[test]
fn dotenv_multiple_filename() {
  Test::new()
    .justfile(
      r#"
        set dotenv-filename := ['.env1', '.env2']

        foo:
          @echo $KEY1-$KEY2-$KEY3
      "#,
    )
    .write(".env1", "KEY1=one\nKEY2=two")
    .write(".env2", "KEY2=override\nKEY3=three")
    .stdout("one-override-three\n")
    .run();
}

#[test]
fn dotenv_multiple_filename_parent() {
  Test::new()
    .justfile(
      r#"
        set dotenv-filename := ['.env1', '.env2']

        foo:
          @echo $KEY1-$KEY2
      "#,
    )
    .write(".env1", "KEY1=parent1")
    .write(".env2", "KEY2=parent2")
    .write(
      "sub/justfile",
      "set dotenv-filename := ['.env1', '.env2']\n@foo:\n echo $KEY1-$KEY2",
    )
    .current_dir("sub")
    .stdout("parent1-parent2\n")
    .run();
}

#[test]
fn dotenv_multiple_path() {
  Test::new()
    .justfile(
      r#"
        set dotenv-path := ['config/.env1', 'config/.env2']

        foo:
          @echo $KEY1-$KEY2-$KEY3
      "#,
    )
    .write("config/.env1", "KEY1=one\nKEY2=two")
    .write("config/.env2", "KEY2=override\nKEY3=three")
    .stdout("one-override-three\n")
    .run();
}

#[test]
fn dotenv_empty_path_array_falls_back_to_dotenv_filename() {
  Test::new()
    .justfile(
      r#"
        set dotenv-path := []
        set dotenv-load

        foo:
          @echo $KEY
      "#,
    )
    .write(".env", "KEY=value")
    .stdout("value\n")
    .run();
}

#[test]
fn dotenv_empty_filename_array() {
  Test::new()
    .justfile(
      r#"
        set dotenv-filename := []

        foo:
          @echo ${KEY:-not_set}
      "#,
    )
    .write(".env", "KEY=value")
    .stdout("not_set\n")
    .run();
}

#[test]
fn dotenv_multiple_filenames_with_variable_expansion() {
  Test::new()
    .justfile(
      r#"
        set dotenv-filename := [".env", x'.env.${ENV:-dev}']

        foo:
          @echo $KEY1$KEY2
      "#,
    )
    .write(".env", "KEY1=default\nKEY2=default")
    .write(".env.dev", "KEY2=dev")
    .write(".env.prod", "KEY2=prod")
    .env("ENV", "prod")
    .stdout("defaultprod\n")
    .run();
}
