use crate::common::*;

#[test]
fn dotenv() {
  let tmp = temptree! {
    ".env": "KEY=ROOT",
    sub: {
      ".env": "KEY=SUB",
      justfile: "default:\n\techo KEY=${KEY:-unset}",
    },
  };

  let binary = executable_path("just");

  let output = Command::new(binary)
    .current_dir(tmp.path())
    .arg("sub/default")
    .output()
    .expect("just invocation failed");

  assert_eq!(output.status.code().unwrap(), 0);

  let stdout = str::from_utf8(&output.stdout).unwrap();
  assert_eq!(stdout, "KEY=unset\n");
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
        echo $NAME
    ",
    )
    .args(&["--dotenv-path", ".env.prod"])
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
        @echo $NAME
    ",
    )
    .tree(tree! {
      subdir: {
        ".env": "NAME=bar"
      }
    })
    .args(&["--dotenv-path", "subdir/.env"])
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
        @echo $NAME
    ",
    )
    .tree(tree! {
      ".env.special": "NAME=bar"
    })
    .args(&["--dotenv-filename", ".env.special"])
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
        @echo $NAME
    ",
    )
    .tree(tree! {
      ".env.special": "NAME=bar"
    })
    .args(&["--dotenv-filename", ".env.special"])
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
        @echo $NAME
    ",
    )
    .tree(tree! {
      subdir: {
        ".env": "NAME=bar"
      }
    })
    .args(&["--dotenv-path", "subdir/.env"])
    .stdout("bar\n")
    .status(EXIT_SUCCESS)
    .run();
}
