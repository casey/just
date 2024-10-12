use super::*;

const JUSTFILE: &str = r#"
foo := `cat data`

linewise bar=`cat data`: shebang
  echo expression: {{foo}}
  echo default: {{bar}}
  echo linewise: `cat data`

shebang:
  #!/usr/bin/env sh
  echo "shebang:" `cat data`
"#;

const DATA: &str = "OK";

const WANT: &str = "shebang: OK\nexpression: OK\ndefault: OK\nlinewise: OK\n";

/// Test that just runs with the correct working directory when invoked with
/// `--justfile` but not `--working-directory`
#[test]
fn justfile_without_working_directory() -> Result<(), Box<dyn Error>> {
  let tmp = temptree! {
    justfile: JUSTFILE,
    data: DATA,
  };

  let output = Command::new(executable_path("just"))
    .arg("--justfile")
    .arg(tmp.path().join("justfile"))
    .output()?;

  if !output.status.success() {
    eprintln!("{:?}", String::from_utf8_lossy(&output.stderr));
    panic!();
  }

  let stdout = String::from_utf8(output.stdout).unwrap();

  assert_eq!(stdout, WANT);

  Ok(())
}

/// Test that just runs with the correct working directory when invoked with
/// `--justfile` but not `--working-directory`, and justfile path has no parent
#[test]
fn justfile_without_working_directory_relative() -> Result<(), Box<dyn Error>> {
  let tmp = temptree! {
    justfile: JUSTFILE,
    data: DATA,
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--justfile")
    .arg("justfile")
    .output()?;

  if !output.status.success() {
    eprintln!("{:?}", String::from_utf8_lossy(&output.stderr));
    panic!();
  }

  let stdout = String::from_utf8(output.stdout).unwrap();

  assert_eq!(stdout, WANT);

  Ok(())
}

/// Test that just invokes commands from the directory in which the justfile is
/// found
#[test]
fn change_working_directory_to_search_justfile_parent() -> Result<(), Box<dyn Error>> {
  let tmp = temptree! {
    justfile: JUSTFILE,
    data: DATA,
    subdir: {},
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path().join("subdir"))
    .output()?;

  if !output.status.success() {
    eprintln!("{:?}", String::from_utf8_lossy(&output.stderr));
    panic!();
  }

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(stdout, WANT);

  Ok(())
}

/// Test that just runs with the correct working directory when invoked with
/// `--justfile` but not `--working-directory`
#[test]
fn justfile_and_working_directory() -> Result<(), Box<dyn Error>> {
  let tmp = temptree! {
    justfile: JUSTFILE,
    sub: {
      data: DATA,
    },
  };

  let output = Command::new(executable_path("just"))
    .arg("--justfile")
    .arg(tmp.path().join("justfile"))
    .arg("--working-directory")
    .arg(tmp.path().join("sub"))
    .output()?;

  if !output.status.success() {
    eprintln!("{:?}", String::from_utf8_lossy(&output.stderr));
    panic!();
  }

  let stdout = String::from_utf8(output.stdout).unwrap();

  assert_eq!(stdout, WANT);

  Ok(())
}

/// Test that just runs with the correct working directory when invoked with
/// `--justfile` but not `--working-directory`
#[test]
fn search_dir_child() -> Result<(), Box<dyn Error>> {
  let tmp = temptree! {
    child: {
      justfile: JUSTFILE,
      data: DATA,
    },
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("child/")
    .output()?;

  if !output.status.success() {
    eprintln!("{:?}", String::from_utf8_lossy(&output.stderr));
    panic!();
  }

  let stdout = String::from_utf8(output.stdout).unwrap();

  assert_eq!(stdout, WANT);

  Ok(())
}

/// Test that just runs with the correct working directory when invoked with
/// `--justfile` but not `--working-directory`
#[test]
fn search_dir_parent() -> Result<(), Box<dyn Error>> {
  let tmp = temptree! {
    child: {
    },
    justfile: JUSTFILE,
    data: DATA,
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path().join("child"))
    .arg("../")
    .output()?;

  if !output.status.success() {
    eprintln!("{:?}", String::from_utf8_lossy(&output.stderr));
    panic!();
  }

  let stdout = String::from_utf8(output.stdout).unwrap();

  assert_eq!(stdout, WANT);

  Ok(())
}

#[test]
fn setting() {
  Test::new()
    .justfile(
      r#"
      set working-directory := 'bar'

      print1:
        echo "$(basename "$PWD")"

      [no-cd]
      print2:
        echo "$(basename "$PWD")"
    "#,
    )
    .current_dir("foo")
    .tree(tree! {
      foo: {},
      bar: {}
    })
    .args(["print1", "print2"])
    .stderr(
      r#"echo "$(basename "$PWD")"
echo "$(basename "$PWD")"
"#,
    )
    .stdout("bar\nfoo\n")
    .run();
}

#[test]
fn no_cd_overrides_setting() {
  Test::new()
    .justfile(
      "
      set working-directory := 'bar'

      [no-cd]
      foo:
        cat bar
    ",
    )
    .current_dir("foo")
    .tree(tree! {
      foo: {
        bar: "hello",
      }
    })
    .stderr("cat bar\n")
    .stdout("hello")
    .run();
}

#[test]
fn working_dir_in_submodule_is_relative_to_module_path() {
  Test::new()
    .write(
      "foo/mod.just",
      "
set working-directory := 'bar'

@foo:
  cat file.txt
",
    )
    .justfile("mod foo")
    .write("foo/bar/file.txt", "FILE")
    .arg("foo")
    .stdout("FILE")
    .run();
}

#[test]
fn working_dir_applies_to_backticks() {
  Test::new()
    .justfile(
      "
        set working-directory := 'foo'

        file := `cat file.txt`

        @foo:
          echo {{ file }}
      ",
    )
    .write("foo/file.txt", "FILE")
    .stdout("FILE\n")
    .run();
}

#[test]
fn working_dir_applies_to_shell_function() {
  Test::new()
    .justfile(
      "
        set working-directory := 'foo'

        file := shell('cat file.txt')

        @foo:
          echo {{ file }}
      ",
    )
    .write("foo/file.txt", "FILE")
    .stdout("FILE\n")
    .run();
}

#[test]
fn working_dir_applies_to_backticks_in_submodules() {
  Test::new()
    .justfile("mod foo")
    .write(
      "foo/mod.just",
      "
set working-directory := 'bar'

file := `cat file.txt`

@foo:
  echo {{ file }}
",
    )
    .arg("foo")
    .write("foo/bar/file.txt", "FILE")
    .stdout("FILE\n")
    .run();
}

#[test]
fn working_dir_applies_to_shell_function_in_submodules() {
  Test::new()
    .justfile("mod foo")
    .write(
      "foo/mod.just",
      "
set working-directory := 'bar'

file := shell('cat file.txt')

@foo:
  echo {{ file }}
",
    )
    .arg("foo")
    .write("foo/bar/file.txt", "FILE")
    .stdout("FILE\n")
    .run();
}

#[test]
fn missing_working_directory_produces_clear_message() {
  Test::new()
    .justfile(
      "
      set working-directory := 'missing'
      default:
        pwd
    ",
    )
    .status(1)
    .stderr_regex(".*Failed to run recipe `default`:\n  Failed to run shell `bash`:\n    .*\n  Failed to set working directory to `.*/missing`.*")
    .run();
}

#[test]
#[cfg(unix)]
fn unusable_working_directory_produces_clear_message() {
  use {fs::Permissions, std::os::unix::fs::PermissionsExt};
  Test::new()
  .justfile(
    "
    set working-directory := 'unusable'
    default:
      pwd
  ",
  )
  .tree(tree! {
    unusable: {}
  })
  .chmod("unusable", Permissions::from_mode(0o000))
  .status(1)
  .stderr_regex(".*Failed to run recipe `default`:\n  Failed to run shell `bash`:\n    .*\n  Failed to set working directory to `.*/unusable`.*")
  .run();
}

#[test]
fn working_directory_is_not_a_directory_produces_clear_message() {
  Test::new()
  .justfile(
    "
    set working-directory := 'unusable'
    default:
      pwd
  ",
  )
  .tree(tree! {
    unusable: "is not a directory"
  })
  .status(1)
  .stderr_regex(".*Failed to run recipe `default`:\n  Failed to run shell `bash`:\n    .*\n  Failed to set working directory to `.*/unusable`.*")
  .run();
}

#[test]
fn missing_working_directory_and_missing_shell_produces_clear_message() {
  Test::new()
    .justfile(
      "
      set working-directory := 'missing'
      default:
        pwd
    ",
    )
    .shell(false)
    .args(["--shell", "NOT_A_REAL_SHELL"])
    .status(1)
    .stderr_regex(".*Failed to run recipe `default`:\n  Failed to run shell `NOT_A_REAL_SHELL`:\n    .*\n  Failed to set working directory to `.*/missing`.*")
    .run();
}

#[test]
#[cfg(unix)]
fn unusable_working_directory_and_missing_shell_produces_clear_message() {
  use {fs::Permissions, std::os::unix::fs::PermissionsExt};
  Test::new()
  .justfile(
    "
    set working-directory := 'unusable'
    default:
      pwd
  ",
  )
  .tree(tree! {
    unusable: {}
  })
  .shell(false)
  .args(["--shell", "NOT_A_REAL_SHELL"])
  .chmod("unusable", Permissions::from_mode(0o000))
  .status(1)
  .stderr_regex(".*Failed to run recipe `default`:\n  Failed to run shell `NOT_A_REAL_SHELL`:\n    .*\n  Failed to set working directory to `.*/unusable`.*")
  .run();
}

#[test]
fn working_directory_is_not_a_directory_and_missing_shell_produces_clear_message() {
  Test::new()
  .justfile(
    "
    set working-directory := 'unusable'
    default:
      pwd
  ",
  )
  .tree(tree! {
    unusable: "is not a directory"
  })
  .shell(false)
  .args(["--shell", "NOT_A_REAL_SHELL"])
  .status(1)
  .stderr_regex(".*Failed to run recipe `default`:\n  Failed to run shell `NOT_A_REAL_SHELL`:\n    .*\n  Failed to set working directory to `.*/unusable`.*")
  .run();
}
