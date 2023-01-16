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
    .run();
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
    .run();
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
    .run();
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
    .run();
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
    .run();
}

#[test]
fn no_choosable_recipes() {
  crate::test::Test::new()
    .arg("--choose")
    .justfile(
      "
        _foo:
          echo foo

        bar BAR:
          echo {{BAR}}
      ",
    )
    .status(EXIT_FAILURE)
    .stderr("error: Justfile contains no choosable recipes.\n")
    .stdout("")
    .run();
}

#[test]
#[ignore]
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
    .run();
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
    .stderr_regex("error: Chooser `/ -cu fzf` invocation failed: .*\n")
    .status(EXIT_FAILURE)
    .shell(false)
    .args(["--shell", "/", "--choose"])
    .run();
}

#[test]
#[cfg(not(windows))]
fn status_error() {
  let tmp = temptree! {
    justfile: "foo:\n echo foo\nbar:\n echo bar\n",
    "exit-2": "#!/usr/bin/env bash\nexit 2\n",
  };

  ("chmod", "+x", tmp.path().join("exit-2")).run();

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
  let fzf = tmp.path().join(format!("fzf{}", env::consts::EXE_SUFFIX));

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
    .current_dir(tmp.path())
    .env("PATH", path)
    .output()
    .unwrap();

  assert_stdout(&output, "foo\n");
}
