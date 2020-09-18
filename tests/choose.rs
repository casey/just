use crate::common::*;

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
  stderr: "Justfile contains no choosable recipes.\n",
  status: EXIT_FAILURE,
}

#[test]
fn default() {
  let tmp = tmptree! {
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
