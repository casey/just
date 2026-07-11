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
    .arg("--choose")
    .env("JUST_CHOOSER", "head -n10")
    .write(
      "bar.just",
      "
        baz:
         echo BAZ
      ",
    )
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
    .stderr("error: justfile contains no choosable recipes\n")
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
      r#"error: chooser `/ -cu fzf --multi --preview 'just --unstable --color always --justfile ".*justfile" --show \{\}'` invocation failed: .*\n"#,
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
  let tmp = tempdir();
  fs::write(
    tmp.path().join("justfile"),
    "foo:\n echo foo\nbar:\n echo bar\n",
  )
  .unwrap();
  fs::write(tmp.path().join("exit-2"), "#!/usr/bin/env bash\nexit 2\n").unwrap();

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

  let output = Command::new(JUST)
    .current_dir(tmp.path())
    .arg("--choose")
    .arg("--chooser")
    .arg("exit-2")
    .env("PATH", path)
    .output()
    .unwrap();

  assert!(
    Regex::new("^error: chooser `exit-2` failed: exit (code|status): 2\n$")
      .unwrap()
      .is_match(str::from_utf8(&output.stderr).unwrap())
  );

  assert_eq!(output.status.code().unwrap(), 2);
}

#[test]
fn cancelled_by_user() {
  if cfg!(windows) {
    return;
  }

  let tmp = tempdir();
  fs::write(
    tmp.path().join("justfile"),
    "foo:\n echo foo\nbar:\n echo bar\n",
  )
  .unwrap();
  fs::write(
    tmp.path().join("chooser"),
    "#!/usr/bin/env bash\nexit 130\n",
  )
  .unwrap();

  let output = Command::new("chmod")
    .arg("+x")
    .arg(tmp.path().join("chooser"))
    .output()
    .unwrap();

  assert!(output.status.success());

  let output = Command::new(JUST)
    .current_dir(tmp.path())
    .arg("--choose")
    .arg("--chooser")
    .arg("./chooser")
    .output()
    .unwrap();

  assert!(output.stderr.is_empty());

  assert!(output.status.success());
}

#[test]
fn chooser_selections_are_processed_separately() {
  Test::new()
    .args(["--choose", "--chooser", "cat"])
    .write(
      "sub.just",
      "
        bar:
         @echo bar
      ",
    )
    .justfile(
      "
        mod sub

        foo *args:
          @echo foo {{args}}
      ",
    )
    .stdin("foo\nsub bar\n")
    .stdout("foo\nbar\n")
    .success();
}

#[test]
fn filter_by_group() {
  Test::new()
    .args(["--choose", "--chooser", "head -n1", "--group", "foo"])
    .justfile(
      "
        a:
          echo A

        [group: 'foo']
        b:
          echo B
      ",
    )
    .stderr("echo B\n")
    .stdout("B\n")
    .success();
}

#[test]
fn default() {
  let tmp = tempdir();
  fs::write(tmp.path().join("justfile"), "foo:\n echo foo\n").unwrap();

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

  let output = Command::new(JUST)
    .arg("--choose")
    .arg("--chooser=fzf")
    .current_dir(tmp.path())
    .env("PATH", path)
    .output()
    .unwrap();

  assert_stdout(&output, "foo\n");
}

#[test]
fn skip_recipes_in_private_modules() {
  Test::new()
    .args(["--choose", "--chooser", "sort"])
    .justfile(
      "
        [private]
        mod foo

        bar:
          @echo bar
      ",
    )
    .write(
      "foo.just",
      "
        baz:
          @echo baz
      ",
    )
    .stdout("bar\n")
    .success();
}

#[test]
fn visit_modules_in_alphabetical_order() {
  Test::new()
    .justfile(
      "
        mod bar
        mod foo
      ",
    )
    .write("bar.just", "baz:\n  @echo bar\n")
    .write("foo.just", "baz:\n  @echo foo\n")
    .args(["--choose", "--chooser", "head -n1"])
    .stdout("bar\n")
    .success();
}

#[cfg(unix)]
#[test]
fn chooser_signal_exit_code_is_not_propagated() {
  Test::new()
    .justfile("foo:\n")
    .args(["--choose", "--chooser", "kill -TERM $$"])
    .stderr("error: chooser `kill -TERM $$` failed: signal: 15 (SIGTERM)\n")
    .status(143);
}
