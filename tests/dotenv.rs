use super::*;

#[test]
fn dotenv() {
  Test::new()
    .justfile("")
    .write(".env", "KEY=ROOT")
    .write("sub/.env", "KEY=SUB")
    .write("sub/justfile", "default:\n\techo KEY=${KEY:-unset}")
    .args(["sub/default"])
    .stdout("KEY=unset\n")
    .stderr("echo KEY=${KEY:-unset}\n")
    .success();
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
    .success();
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
    .success();
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
    .success();
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
    .success();
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
    .stderr("error: dotenv file not found\n")
    .failure();
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
    .success();
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
    .success();
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
    .success();
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
    .success();
}

#[test]
fn can_set_dotenv_filename_from_justfile() {
  Test::new()
    .justfile(
      "
        set dotenv-filename := '.env.special'

        foo:
          @echo $JUST_TEST_VARIABLE
      ",
    )
    .tree(tree! {
      ".env.special": "JUST_TEST_VARIABLE=bar"
    })
    .stdout("bar\n")
    .success();
}

#[test]
fn can_set_dotenv_path_from_justfile() {
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
        ".env": "JUST_TEST_VARIABLE=bar"
      }
    })
    .stdout("bar\n")
    .success();
}

#[test]
fn program_argument_has_priority_for_dotenv_filename() {
  Test::new()
    .justfile(
      "
        set dotenv-filename := '.env.special'

        foo:
          @echo $JUST_TEST_VARIABLE
      ",
    )
    .tree(tree! {
      ".env.special": "JUST_TEST_VARIABLE=bar",
      ".env.superspecial": "JUST_TEST_VARIABLE=baz"
    })
    .args(["--dotenv-filename", ".env.superspecial"])
    .stdout("baz\n")
    .success();
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
    .success();
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
    .success();
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
    .success();
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
    .success();
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
    .success();
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
    .success();
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
    .success();
}

#[test]
fn dotenv_env_var_default_no_override() {
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
    .success();
}

#[test]
fn dotenv_env_var_override() {
  Test::new()
    .justfile(
      "
        set dotenv-load
        set dotenv-override := true
        echo:
          echo $DOTENV_KEY
      ",
    )
    .write(".env", "DOTENV_KEY=dotenv-value")
    .env("DOTENV_KEY", "not-the-dotenv-value")
    .stdout("dotenv-value\n")
    .stderr("echo $DOTENV_KEY\n")
    .success();
}

#[test]
fn dotenv_env_var_override_no_load() {
  Test::new()
    .justfile(
      "
        set dotenv-override := true
        echo:
          echo $DOTENV_KEY
      ",
    )
    .write(".env", "DOTENV_KEY=dotenv-value")
    .env("DOTENV_KEY", "not-the-dotenv-value")
    .stdout("dotenv-value\n")
    .stderr("echo $DOTENV_KEY\n")
    .success();
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
    .success();
}

#[test]
fn dotenv_path_does_not_override_dotenv_file() {
  Test::new()
    .justfile("")
    .write(".env", "KEY=ROOT")
    .write(
      "sub/justfile",
      "set dotenv-path := '.'\n@foo:\n echo ${KEY}",
    )
    .current_dir("sub")
    .stdout("ROOT\n")
    .success();
}

#[test]
fn directory_is_ignored() {
  Test::new()
    .justfile(
      "
        set dotenv-load

        foo:
          echo ${DOTENV_KEY:-unset}
      ",
    )
    .create_dir(".env")
    .stdout("unset\n")
    .stderr("echo ${DOTENV_KEY:-unset}\n")
    .success();
}

#[test]
fn fifo() {
  if !cfg!(unix) {
    return;
  }

  let test = Test::new();

  let fifo = test.tempdir.path().join(".env");

  let status = Command::new("mkfifo").arg(&fifo).status().unwrap();

  assert!(status.success());

  thread::spawn(move || {
    fs::write(fifo, "DOTENV_KEY=foo\n").unwrap();
  });

  test
    .justfile(
      "
        set dotenv-load

        bar:
          echo $DOTENV_KEY
      ",
    )
    .stdout("foo\n")
    .stderr("echo $DOTENV_KEY\n")
    .success();
}

#[test]
fn error_message() {
  Test::new()
    .write(".env", "FOO=bar baz")
    .justfile(
      "
        set dotenv-load

        foo:
      ",
    )
    .stderr_regex(r"error: failed to load environment file from `.*\.env`: .*")
    .failure();
}

#[test]
fn path_list_last_wins() {
  Test::new()
    .justfile(
      "
        set lists
        set dotenv-path := ['foo.env', 'bar.env']

        @foo:
          echo $KEY
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .write("foo.env", "KEY=foo")
    .write("bar.env", "KEY=bar")
    .stdout("bar\n")
    .success();
}

#[test]
fn filename_list_loads_all_in_directory() {
  Test::new()
    .justfile(
      "
        set lists
        set dotenv-filename := ['.env.foo', '.env.bar']

        @foo:
          echo $FOO $BAR $SHARED
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .write(".env.foo", "FOO=foo\nSHARED=from-foo")
    .write(".env.bar", "BAR=bar\nSHARED=from-bar")
    .stdout("foo bar from-bar\n")
    .success();
}

#[test]
fn filename_list_stops_at_first_directory() {
  Test::new()
    .justfile("")
    .write(
      "sub/justfile",
      "set lists\nset dotenv-filename := ['.env.foo', '.env.bar']\n@foo:\n\techo \"${FOO:-unset} ${BAR:-unset}\"",
    )
    .write("sub/.env.foo", "FOO=foo")
    .write(".env.bar", "BAR=bar")
    .env("JUST_UNSTABLE", "1")
    .current_dir("sub")
    .args(["foo"])
    .stdout("foo unset\n")
    .success();
}

#[test]
fn path_list_falls_through_to_filename_search() {
  Test::new()
    .justfile(
      "
        set lists
        set dotenv-path := ['foo.env', 'bar.env']

        @foo:
          echo $KEY
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .write(".env", "KEY=foo")
    .stdout("foo\n")
    .success();
}

#[test]
fn list_override() {
  Test::new()
    .justfile(
      "
        set lists
        set dotenv-override
        set dotenv-path := ['foo.env', 'bar.env']

        @foo:
          echo $KEY
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .env("KEY", "environment")
    .write("foo.env", "KEY=foo")
    .write("bar.env", "KEY=bar")
    .stdout("bar\n")
    .success();
}

#[test]
fn list_does_not_override_environment() {
  Test::new()
    .justfile(
      "
        set lists
        set dotenv-path := ['foo.env', 'bar.env']

        @foo:
          echo $KEY
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .env("KEY", "environment")
    .write("foo.env", "KEY=foo")
    .write("bar.env", "KEY=bar")
    .stdout("environment\n")
    .success();
}

#[test]
fn required_satisfied_by_one_file() {
  Test::new()
    .justfile(
      "
        set lists
        set dotenv-required
        set dotenv-path := ['missing.env', 'present.env']

        @foo:
          echo $KEY
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .write("present.env", "KEY=foo")
    .stdout("foo\n")
    .success();
}

#[test]
fn empty_filename_list_is_unset() {
  Test::new()
    .justfile(
      "
        set lists
        set dotenv-filename := []

        @foo:
          echo ${KEY:-unset}
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .write(".env", "KEY=foo")
    .stdout("unset\n")
    .success();
}

#[test]
fn filename_list_requires_lists_setting() {
  Test::new()
    .justfile(
      "
        set dotenv-filename := ['foo', 'bar']

        foo:
      ",
    )
    .stderr(
      "
        error: list literals require `set lists`
         ——▶ justfile:1:24
          │
        1 │ set dotenv-filename := ['foo', 'bar']
          │                        ^
      ",
    )
    .failure();
}

#[test]
fn path_argument_list() {
  Test::new()
    .justfile(
      "
        set lists

        @foo:
          echo $KEY
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .write("foo.env", "KEY=foo")
    .write("bar.env", "KEY=bar")
    .args(["--dotenv-path", "foo.env", "--dotenv-path", "bar.env"])
    .stdout("bar\n")
    .success();
}

#[test]
fn filename_argument_list() {
  Test::new()
    .justfile(
      "
        set lists

        @foo:
          echo $FOO $BAR
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .write(".env.foo", "FOO=foo")
    .write(".env.bar", "BAR=bar")
    .args([
      "--dotenv-filename",
      ".env.foo",
      "--dotenv-filename",
      ".env.bar",
    ])
    .stdout("foo bar\n")
    .success();
}

#[test]
fn multiple_arguments_require_lists_setting() {
  Test::new()
    .justfile(
      "
        @foo:
          echo $KEY
      ",
    )
    .args(["--dotenv-path", "foo.env", "--dotenv-path", "bar.env"])
    .stderr(
      "error: multiple `--dotenv-filename` or `--dotenv-path` arguments require `set lists`\n",
    )
    .failure();
}

#[test]
fn command_setting() {
  Test::new()
    .justfile(
      "
        set dotenv-command := 'echo KEY=command'

        @foo:
          echo $KEY
      ",
    )
    .stdout("command\n")
    .success();
}

#[test]
fn command_option() {
  Test::new()
    .justfile(
      "
        @foo:
          echo $KEY
      ",
    )
    .args(["--dotenv-command", "echo KEY=command"])
    .stdout("command\n")
    .success();
}

#[test]
fn command_option_multiple() {
  Test::new()
    .justfile(
      "
        @foo:
          echo $FOO $BAZ
      ",
    )
    .args([
      "--dotenv-command",
      "echo FOO=bar",
      "--dotenv-command",
      "echo BAZ=qux",
    ])
    .stdout("bar qux\n")
    .success();
}

#[test]
fn command_list_runs_each_and_merges() {
  Test::new()
    .justfile(
      "
        set lists
        set dotenv-command := ['echo FOO=bar', 'echo BAZ=qux']

        @foo:
          echo $FOO $BAZ
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("bar qux\n")
    .success();
}

#[test]
fn command_list_last_wins() {
  Test::new()
    .justfile(
      "
        set lists
        set dotenv-command := ['echo KEY=foo', 'echo KEY=bar']

        @foo:
          echo $KEY
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
    .success();
}

#[test]
fn command_does_not_override_environment() {
  Test::new()
    .justfile(
      "
        set dotenv-command := 'echo KEY=command'

        @foo:
          echo $KEY
      ",
    )
    .env("KEY", "environment")
    .stdout("environment\n")
    .success();
}

#[test]
fn command_with_override() {
  Test::new()
    .justfile(
      "
        set dotenv-command := 'echo KEY=command'
        set dotenv-override := true

        @foo:
          echo $KEY
      ",
    )
    .env("KEY", "environment")
    .stdout("command\n")
    .success();
}

#[test]
fn command_failure() {
  Test::new()
    .justfile(
      "
        set dotenv-command := 'exit 1'

        foo:
      ",
    )
    .stderr("error: dotenv command `exit 1` failed: process exited with status code 1\n")
    .failure();
}

#[test]
fn command_conflicts_with_dotenv_path_setting() {
  Test::new()
    .justfile(
      "
        set dotenv-command := 'true'
        set dotenv-path := 'foo'

        foo:
      ",
    )
    .stderr(
      "
        error: `dotenv-command` set on line 1 is incompatible with `dotenv-path`
         ——▶ justfile:2:5
          │
        2 │ set dotenv-path := 'foo'
          │     ^^^^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn command_conflicts_with_dotenv_required_setting() {
  Test::new()
    .justfile(
      "
        set dotenv-required
        set dotenv-command := 'true'

        foo:
      ",
    )
    .stderr(
      "
        error: `dotenv-required` set on line 1 is incompatible with `dotenv-command`
         ——▶ justfile:2:5
          │
        2 │ set dotenv-command := 'true'
          │     ^^^^^^^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn command_composes_with_dotenv_override_setting() {
  Test::new()
    .justfile(
      "
        set dotenv-override := true
        set dotenv-command := 'echo KEY=command'

        @foo:
          echo $KEY
      ",
    )
    .env("KEY", "environment")
    .stdout("command\n")
    .success();
}
