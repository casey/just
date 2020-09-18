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
  name: default,
  justfile: "
    fzf:
      @Ran `fzf` recipe.
  ",
  args: ("--choose", "--shell", "echo", "--clear-shell-args", "--chooser", "echo"),
  stdout: "Ran `fzf` recipe.\n",
  stderr: "",
  shell: false,
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
