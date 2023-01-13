use super::*;
use temptree::temptree;

#[test]
fn include_fails_without_unstable() {
  let justfile_contents = r#"
        # Include include.justfile
        !include ./include.justfile

        recipe_a: recipe_b
            echo "A"
        "#;

  let include_justfile_contents = unindent(
    r#"
        recipe_b:
            echo "B"
        "#,
  );

  let tmp = temptree! {
      "include.justfile": include_justfile_contents,
  };

  Test::with_tempdir(tmp)
    .justfile(justfile_contents)
    .status(EXIT_FAILURE)
    .stderr("error: Expected character `=`\n  |\n2 | !include ./include.justfile\n  |  ^\n")
    .run();
}

#[test]
fn include_succeeds_with_unstable() {
  let justfile_contents = r#"
        # !include should work with trailing spaces
        !include ./include.justfile     

        recipe_a: recipe_b
            @echo "A"
        "#;

  let include_justfile_contents = unindent(
    r#"
        recipe_b:
            @echo "B"
        "#,
  );

  let tmp = temptree! {
      "include.justfile": include_justfile_contents,
  };

  Test::with_tempdir(tmp)
    .justfile(justfile_contents)
    .arg("--unstable")
    .arg("recipe_a")
    .status(EXIT_SUCCESS)
    .stdout("B\nA\n")
    .run();
}

#[test]
fn include_directive_with_no_path() {
  let justfile_contents = r#"

    !include

    recipe_a:
        @echo "hello"
    "#;

  let tmp = temptree! {
      "include.justfile": "#empty justfile",
  };

  let mut path = tmp.path().to_owned();
  path.push("justfile");

  Test::with_tempdir(tmp)
    .justfile(justfile_contents)
    .arg("--unstable")
    .status(EXIT_FAILURE)
    .stderr(&format!(
      "error: !include statement in {} line 2 has no argument\n",
      path.display()
    ))
    .run();
}

#[test]
fn trailing_include() {
  let justfile_contents = r#"

    recipe_b:
        @echo "B"

    !include ./include.justfile

    recipe_a:
        @echo "hello"
    "#;

  let tmp = temptree! {
      "include.justfile": "#empty justfile",
  };

  let mut path = tmp.path().to_owned();
  path.push("justfile");

  Test::with_tempdir(tmp)
    .justfile(justfile_contents)
    .arg("--unstable")
    .status(EXIT_FAILURE)
    .stderr(format!(
      "error: Expected character `=`\n  |\n5 | !include ./include.justfile\n  |  ^\n",
    ))
    .run();
}
