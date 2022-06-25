use super::*;

test! {
  name: env,
  justfile: "
    foo:
      echo foo

    bar:
      echo bar
  ",
  args: ("--choose"),
  env: {
    "JUST_CHOOSER": "head -n1",
  },
  stdout: "bar\n",
  stderr: "echo bar\n",
}

test! {
  name: chooser,
  justfile: "
    foo:
      echo foo

    bar:
      echo bar
  ",
  args: ("--choose", "--chooser", "head -n1"),
  stdout: "bar\n",
  stderr: "echo bar\n",
}

test! {
  name: override_variable,
  justfile: "
    baz := 'A'

    foo:
      echo foo

    bar:
      echo {{baz}}
  ",
  args: ("--choose", "baz=B"),
  env: {
    "JUST_CHOOSER": "head -n1",
  },
  stdout: "B\n",
  stderr: "echo B\n",
}

test! {
  name: skip_private_recipes,
  justfile: "
    foo:
      echo foo

    _bar:
      echo bar
  ",
  args: ("--choose"),
  env: {
    "JUST_CHOOSER": "head -n1",
  },
  stdout: "foo\n",
  stderr: "echo foo\n",
}

test! {
  name: skip_recipes_that_require_arguments,
  justfile: "
    foo:
      echo foo

    bar BAR:
      echo {{BAR}}
  ",
  args: ("--choose"),
  env: {
    "JUST_CHOOSER": "head -n1",
  },
  stdout: "foo\n",
  stderr: "echo foo\n",
}

test! {
  name: no_choosable_recipes,
  justfile: "
    _foo:
      echo foo

    bar BAR:
      echo {{BAR}}
  ",
  args: ("--choose"),
  stdout: "",
  stderr: "error: Justfile contains no choosable recipes.\n",
  status: EXIT_FAILURE,
}

test! {
  name: multiple_recipes,
  justfile: "
    foo:
      echo foo

    bar:
      echo bar
  ",
  args: ("--choose", "--chooser", "echo foo bar"),
  stdout: "foo\nbar\n",
  stderr: "echo foo\necho bar\n",
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
    .stderr_regex("error: Chooser `/ -cu fzf` invocation failed: .*")
    .status(EXIT_FAILURE)
    .shell(false)
    .args(&["--shell", "/", "--choose"])
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
