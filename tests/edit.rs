use super::*;

const JUSTFILE: &str = "Yooooooo, hopefully this never becomes valid syntax.";

/// Test that --edit doesn't require a valid justfile
#[test]
fn invalid_justfile() {
  let tmp = temptree! {
    justfile: JUSTFILE,
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .output()
    .unwrap();

  assert!(!output.status.success());

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--edit")
    .env("VISUAL", "cat")
    .output()
    .unwrap();

  assert_stdout(&output, JUSTFILE);
}

#[test]
fn invoke_error() {
  let tmp = temptree! {
    justfile: JUSTFILE,
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .output()
    .unwrap();

  assert!(!output.status.success());

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--edit")
    .env("VISUAL", "/")
    .output()
    .unwrap();

  assert_eq!(
    String::from_utf8_lossy(&output.stderr),
    if cfg!(windows) {
      "error: Editor `/` invocation failed: program path has no file name\n"
    } else {
      "error: Editor `/` invocation failed: Permission denied (os error 13)\n"
    }
  );
}

#[test]
#[cfg(not(windows))]
fn status_error() {
  let tmp = temptree! {
    justfile: JUSTFILE,
    "exit-2": "#!/usr/bin/env bash\nexit 2\n",
  };

  ("chmod", "+x", tmp.path().join("exit-2")).run();

  let path = env::join_paths(
    iter::once(tmp.path().to_owned()).chain(env::split_paths(&env::var_os("PATH").unwrap())),
  )
  .unwrap();

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--edit")
    .env("PATH", path)
    .env("VISUAL", "exit-2")
    .output()
    .unwrap();

  assert!(
    Regex::new("^error: Editor `exit-2` failed: exit (code|status): 2\n$")
      .unwrap()
      .is_match(str::from_utf8(&output.stderr).unwrap())
  );

  assert_eq!(output.status.code().unwrap(), 2);
}

/// Test that editor is $VISUAL, $EDITOR, or "vim" in that order
#[test]
fn editor_precedence() {
  let tmp = temptree! {
    justfile: JUSTFILE,
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--edit")
    .env("VISUAL", "cat")
    .env("EDITOR", "this-command-doesnt-exist")
    .output()
    .unwrap();

  assert_stdout(&output, JUSTFILE);

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--edit")
    .env_remove("VISUAL")
    .env("EDITOR", "cat")
    .output()
    .unwrap();

  assert_stdout(&output, JUSTFILE);

  let cat = which("cat").unwrap();
  let vim = tmp.path().join(format!("vim{EXE_SUFFIX}"));

  #[cfg(unix)]
  std::os::unix::fs::symlink(cat, vim).unwrap();

  #[cfg(windows)]
  std::os::windows::fs::symlink_file(cat, vim).unwrap();

  let path = env::join_paths(
    iter::once(tmp.path().to_owned()).chain(env::split_paths(&env::var_os("PATH").unwrap())),
  )
  .unwrap();

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--edit")
    .env("PATH", path)
    .env_remove("VISUAL")
    .env_remove("EDITOR")
    .output()
    .unwrap();

  assert_stdout(&output, JUSTFILE);
}

/// Test that editor working directory is the same as edited justfile
#[cfg(unix)]
#[test]
fn editor_working_directory() {
  let tmp = temptree! {
    justfile: JUSTFILE,
    child: {},
    editor: "#!/usr/bin/env sh\ncat $1\npwd",
  };

  let editor = tmp.path().join("editor");

  let permissions = std::os::unix::fs::PermissionsExt::from_mode(0o700);
  fs::set_permissions(&editor, permissions).unwrap();

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path().join("child"))
    .arg("--edit")
    .env("VISUAL", &editor)
    .output()
    .unwrap();

  let want = format!(
    "{JUSTFILE}{}\n",
    tmp.path().canonicalize().unwrap().display()
  );

  assert_stdout(&output, &want);
}
