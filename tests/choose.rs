use super::*;

#[test]
fn env() {
  Test::new()
    .arg("--choose")
    .env("JUST_CHOOSER", "head -n1")
    .justfile(
      "
        foo:
          echo foo

        bar:
          echo bar
      ",
    )
    .stderr("echo bar\n")
    .stdout("bar\n")
    .success();
}

#[test]
fn chooser() {
  Test::new()
    .arg("--choose")
    .arg("--chooser")
    .arg("head -n1")
    .justfile(
      "
        foo:
          echo foo

        bar:
          echo bar
      ",
    )
    .stderr("echo bar\n")
    .stdout("bar\n")
    .success();
}

#[test]
fn override_variable() {
  Test::new()
    .arg("--choose")
    .arg("baz=B")
    .env("JUST_CHOOSER", "head -n1")
    .justfile(
      "
        baz := 'A'

        foo:
          echo foo

        bar:
          echo {{baz}}
      ",
    )
    .stderr("echo B\n")
    .stdout("B\n")
    .success();
}

#[test]
fn skip_private_recipes() {
  Test::new()
    .arg("--choose")
    .env("JUST_CHOOSER", "head -n1")
    .justfile(
      "
        foo:
          echo foo

        _bar:
          echo bar
      ",
    )
    .stderr("echo foo\n")
    .stdout("foo\n")
    .success();
}

#[test]
fn recipes_in_submodules_can_be_chosen() {
  Test::new()
    .args(["--unstable", "--choose"])
    .env("JUST_CHOOSER", "head -n10")
    .write("bar.just", "baz:\n echo BAZ")
    .justfile(
      "
        mod bar
      ",
    )
    .stderr("echo BAZ\n")
    .stdout("BAZ\n")
    .success();
}

#[test]
fn skip_recipes_that_require_arguments() {
  Test::new()
    .arg("--choose")
    .env("JUST_CHOOSER", "head -n1")
    .justfile(
      "
        foo:
          echo foo

        bar BAR:
          echo {{BAR}}
      ",
    )
    .stderr("echo foo\n")
    .stdout("foo\n")
    .success();
}

#[test]
fn no_choosable_recipes() {
  Test::new()
    .arg("--choose")
    .justfile(
      "
        _foo:
          echo foo

        bar BAR:
          echo {{BAR}}
      ",
    )
    .stderr("error: Justfile contains no choosable recipes.\n")
    .failure();
}

#[test]
fn multiple_recipes() {
  Test::new()
    .arg("--choose")
    .arg("--chooser")
    .arg("echo foo bar")
    .justfile(
      "
        foo:
          echo foo

        bar:
          echo bar
      ",
    )
    .stderr("echo foo\necho bar\n")
    .stdout("foo\nbar\n")
    .success();
}

#[test]
fn invoke_error_function() {
  Test::new()
    .justfile(
      "
        foo:
          echo foo

        bar:
          echo bar
      ",
    )
    .stderr_regex(
      r#"error: Chooser `/ -cu fzf --multi --preview 'just --unstable --color always --justfile ".*justfile" --show \{\}'` invocation failed: .*\n"#,
    )
    .shell(false)
    .args(["--shell", "/", "--choose"])
    .failure();
}

#[test]
fn status_error() {
  if cfg!(windows) {
    return;
  }
  let tmp = temptree! {
    justfile: "foo:\n echo foo\nbar:\n echo bar\n",
    "exit-2": "#!/usr/bin/env bash\nexit 2\n",
  };

  let output = Command::new("chmod")
    .arg("+x")
    .arg(tmp.path().join("exit-2"))
    .output()
    .unwrap();

  assert!(output.status.success());

  let path = env::join_paths(
    iter::once(tmp.path().to_owned()).chain(env::split_paths(&env::var_os("PATH").unwrap())),
  )
  .unwrap();

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--choose")
    .arg("--chooser")
    .arg("exit-2")
    .env("PATH", path)
    .output()
    .unwrap();

  assert!(
    Regex::new("^error: Chooser `exit-2` failed: exit (code|status): 2\n$")
      .unwrap()
      .is_match(str::from_utf8(&output.stderr).unwrap())
  );

  assert_eq!(output.status.code().unwrap(), 2);
}

#[test]
fn default() {
  let tmp = temptree! {
    justfile: "foo:\n echo foo\n",
  };

  let cat = which("cat").unwrap();
  let fzf = tmp.path().join(format!("fzf{EXE_SUFFIX}"));

  #[cfg(unix)]
  std::os::unix::fs::symlink(cat, fzf).unwrap();

  #[cfg(windows)]
  std::os::windows::fs::symlink_file(cat, fzf).unwrap();

  let path = env::join_paths(
    iter::once(tmp.path().to_owned()).chain(env::split_paths(&env::var_os("PATH").unwrap())),
  )
  .unwrap();

  let output = Command::new(executable_path("just"))
    .arg("--choose")
    .arg("--chooser=fzf")
    .current_dir(tmp.path())
    .env("PATH", path)
    .output()
    .unwrap();

  assert_stdout(&output, "foo\n");
}
