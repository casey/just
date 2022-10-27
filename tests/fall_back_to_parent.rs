use super::*;

#[test]
fn runs_recipe_in_parent_if_not_found_in_current() {
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
    .args(&["--unstable", "foo"])
    .current_dir("bar")
    .stderr(format!(
      "
      Trying ..{}justfile
      echo root
    ",
      MAIN_SEPARATOR
    ))
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
    .args(&["--unstable", "foo"])
    .current_dir("bar")
    .stderr(format!(
      "
      Trying ..{}justfile
      echo root
    ",
      MAIN_SEPARATOR
    ))
    .stdout("root\n")
    .run();
}

#[test]
fn print_error_from_parent_if_recipe_not_found_in_current() {
  Test::new()
    .tree(tree! {
      bar: {
        justfile: "
          baz:
            echo subdir
        "
      }
    })
    .justfile("foo:\n echo {{bar}}")
    .args(&["--unstable", "foo"])
    .current_dir("bar")
    .stderr(format!(
      "
      Trying ..{}justfile
      error: Variable `bar` not defined
        |
      2 |  echo {{{{bar}}}}
        |         ^^^
    ",
      MAIN_SEPARATOR
    ))
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn requires_unstable() {
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
    .args(&["foo"])
    .current_dir("bar")
    .status(EXIT_FAILURE)
    .stderr("error: Justfile does not contain recipe `foo`.\n")
    .run();
}

#[test]
#[ignore]
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
    .args(&["--unstable", "foo"])
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
    .args(&["--unstable", "./foo"])
    .stdout("root\n")
    .stderr(format!(
      "
      Trying ..{}justfile
      echo root
    ",
      MAIN_SEPARATOR
    ))
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
    .args(&["--unstable", "--justfile", "justfile", "foo"])
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
    .args(&[
      "--unstable",
      "--justfile",
      "justfile",
      "--working-directory",
      ".",
      "foo",
    ])
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
    .args(&["--unstable", "foo"])
    .current_dir("bar")
    .status(EXIT_FAILURE)
    .stderr(format!(
      "
      Trying ..{}justfile
      error: Justfile does not contain recipe `foo`.
    ",
      MAIN_SEPARATOR,
    ))
    .run();
}

#[test]
fn multiple_levels_of_fallback_work() {
  Test::new()
    .tree(tree! {
      a: {
        b: {
          justfile: "
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
    .args(&["--unstable", "baz"])
    .current_dir("a/b")
    .stdout("root\n")
    .stderr(format!(
      "
      Trying ..{}justfile
      Trying ..{}..{}justfile
      echo root
    ",
      MAIN_SEPARATOR, MAIN_SEPARATOR, MAIN_SEPARATOR
    ))
    .run();
}

#[test]
fn stop_fallback_when_fallback_is_false() {
  Test::new()
    .tree(tree! {
      a: {
        b: {
          justfile: "
            foo:
              echo subdir
          "
        },
        justfile: "
          set fallback := false
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
    .args(&["--unstable", "baz"])
    .current_dir("a/b")
    .stderr(format!(
      "
      Trying ..{}justfile
      error: Justfile does not contain recipe `baz`.
      Did you mean `bar`?
    ",
      MAIN_SEPARATOR
    ))
    .status(EXIT_FAILURE)
    .run();
}
