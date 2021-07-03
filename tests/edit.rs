use crate::common::*;

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
  let vim = tmp.path().join(format!("vim{}", env::consts::EXE_SUFFIX));

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
  std::fs::set_permissions(&editor, permissions).unwrap();

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path().join("child"))
    .arg("--edit")
    .env("VISUAL", &editor)
    .output()
    .unwrap();

  let want = format!(
    "{}{}\n",
    JUSTFILE,
    tmp.path().canonicalize().unwrap().display()
  );

  assert_stdout(&output, &want);
}
