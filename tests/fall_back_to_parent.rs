use crate::common::*;

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
    .stderr(
      "
      Trying ../justfile
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
          baz:
            echo subdir
        "
      }
    })
    .justfile("foo:\n echo {{bar}}")
    .args(&["--unstable", "foo"])
    .current_dir("bar")
    .stderr(
      "
      Trying ../justfile
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
fn doesnt_work_with_search_directory() {
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
    .current_dir("bar")
    .status(EXIT_FAILURE)
    .stderr("error: Justfile does not contain recipe `foo`.\n")
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
    .stderr(
      "
      Trying ../justfile
      error: Justfile does not contain recipe `foo`.
    ",
    )
    .run();
}
