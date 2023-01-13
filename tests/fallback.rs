use super::*;

#[test]
fn fallback_from_subdir_bugfix() {
  Test::new()
    .write(
      "sub/justfile",
      unindent(
        "
        set fallback

        @default:
          echo foo
      ",
      ),
    )
    .args(["sub/default"])
    .stdout("foo\n")
    .run();
}

#[test]
fn fallback_from_subdir_message() {
  Test::new()
    .justfile("bar:\n echo bar")
    .write(
      "sub/justfile",
      unindent(
        "
        set fallback

        @foo:
          echo foo
      ",
      ),
    )
    .args(["sub/bar"])
    .stderr(path("echo bar\n"))
    .stdout("bar\n")
    .run();
}

#[test]
fn fallback_from_subdir_verbose_message() {
  Test::new()
    .justfile("bar:\n echo bar")
    .write(
      "sub/justfile",
      unindent(
        "
        set fallback

        @foo:
          echo foo
      ",
      ),
    )
    .args(["--verbose", "sub/bar"])
    .stderr(path(
      "
      Trying ../justfile
      ===> Running recipe `bar`...
      echo bar
      ",
    ))
    .stdout("bar\n")
    .run();
}

#[test]
fn runs_recipe_in_parent_if_not_found_in_current() {
  Test::new()
    .tree(tree! {
      bar: {
        justfile: "
          set fallback := true

          baz:
            echo subdir
        "
      }
    })
    .justfile(
      "
      foo:
        echo root
    ",
    )
    .args(["foo"])
    .current_dir("bar")
    .stderr(
      "
      echo root
    ",
    )
    .stdout("root\n")
    .run();
}

#[test]
fn setting_accepts_value() {
  Test::new()
    .tree(tree! {
      bar: {
        justfile: "
          set fallback := true

          baz:
            echo subdir
        "
      }
    })
    .justfile(
      "
      foo:
        echo root
    ",
    )
    .args(["foo"])
    .current_dir("bar")
    .stderr(
      "
      echo root
    ",
    )
    .stdout("root\n")
    .run();
}

#[test]
fn print_error_from_parent_if_recipe_not_found_in_current() {
  Test::new()
    .tree(tree! {
      bar: {
        justfile: "
          set fallback := true

          baz:
            echo subdir
        "
      }
    })
    .justfile("foo:\n echo {{bar}}")
    .args(["foo"])
    .current_dir("bar")
    .stderr(
      "
      error: Variable `bar` not defined
        |
      2 |  echo {{bar}}
        |         ^^^
    ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn requires_setting() {
  Test::new()
    .tree(tree! {
      bar: {
        justfile: "
          baz:
            echo subdir
        "
      }
    })
    .justfile(
      "
      foo:
        echo root
    ",
    )
    .args(["foo"])
    .current_dir("bar")
    .status(EXIT_FAILURE)
    .stderr("error: Justfile does not contain recipe `foo`.\n")
    .run();
}

#[test]
fn works_with_provided_search_directory() {
  Test::new()
    .tree(tree! {
      bar: {
        justfile: "
          set fallback := true

          baz:
            echo subdir
        "
      }
    })
    .justfile(
      "
      foo:
        echo root
    ",
    )
    .args(["./foo"])
    .stdout("root\n")
    .stderr(
      "
      echo root
    ",
    )
    .current_dir("bar")
    .run();
}

#[test]
fn doesnt_work_with_justfile() {
  Test::new()
    .tree(tree! {
      bar: {
        justfile: "
          baz:
            echo subdir
        "
      }
    })
    .justfile(
      "
      foo:
        echo root
    ",
    )
    .args(["--justfile", "justfile", "foo"])
    .current_dir("bar")
    .status(EXIT_FAILURE)
    .stderr("error: Justfile does not contain recipe `foo`.\n")
    .run();
}

#[test]
fn doesnt_work_with_justfile_and_working_directory() {
  Test::new()
    .tree(tree! {
      bar: {
        justfile: "
          baz:
            echo subdir
        "
      }
    })
    .justfile(
      "
      foo:
        echo root
    ",
    )
    .args(["--justfile", "justfile", "--working-directory", ".", "foo"])
    .current_dir("bar")
    .status(EXIT_FAILURE)
    .stderr("error: Justfile does not contain recipe `foo`.\n")
    .run();
}

#[test]
fn prints_correct_error_message_when_recipe_not_found() {
  Test::new()
    .tree(tree! {
      bar: {
        justfile: "
          set fallback := true

          bar:
            echo subdir
        "
      }
    })
    .justfile(
      "
      bar:
        echo root
    ",
    )
    .args(["foo"])
    .current_dir("bar")
    .status(EXIT_FAILURE)
    .stderr(
      "
      error: Justfile does not contain recipe `foo`.
    ",
    )
    .run();
}

#[test]
fn multiple_levels_of_fallback_work() {
  Test::new()
    .tree(tree! {
      a: {
        b: {
          justfile: "
            set fallback := true

            foo:
              echo subdir
          "
        },
        justfile: "
          set fallback := true

          bar:
            echo subdir
        "
      }
    })
    .justfile(
      "
      baz:
        echo root
    ",
    )
    .args(["baz"])
    .current_dir("a/b")
    .stdout("root\n")
    .stderr(
      "
      echo root
    ",
    )
    .run();
}

#[test]
fn stop_fallback_when_fallback_is_false() {
  Test::new()
    .tree(tree! {
      a: {
        b: {
          justfile: "
            set fallback := true

            foo:
              echo subdir
          "
        },
        justfile: "
          bar:
            echo subdir
        "
      }
    })
    .justfile(
      "
      baz:
        echo root
    ",
    )
    .args(["baz"])
    .current_dir("a/b")
    .stderr(
      "
      error: Justfile does not contain recipe `baz`.
      Did you mean `bar`?
    ",
    )
    .status(EXIT_FAILURE)
    .run();
}
