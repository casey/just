use super::*;

#[test]
fn single_justfile_with_slash_syntax() {
  let justfile_contents = unindent(
    r#"
    recipe_a:
        echo "A"

    recipe_b:
        echo "B"
  "#,
  );
  let tmp = temptree! {
      subdir: {
          justfile: justfile_contents
      }
  };

  for arg_list in [
    ["subdir/recipe_a", "recipe_b"],
    ["subdir/recipe_a", "subdir/recipe_b"],
  ] {
    let mut command = Command::new(executable_path("just"));
    command.current_dir(tmp.path());

    for arg in arg_list {
      command.arg(arg);
    }

    let output = command.output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert_eq!(stdout, "A\nB\n");
  }
}

#[test]
fn multiple_justfiles_with_slash_syntax() {
  let justfile_contents1 = unindent(
    r#"
    recipe_a:
        echo "A"

  "#,
  );
  let justfile_contents2 = unindent(
    r#"
    recipe_b:
        echo "B"
      "#,
  );
  let tmp = temptree! {
      subdir: {
          justfile: justfile_contents1
      },
      subdir2: {
          justfile: justfile_contents2
      }
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("subdir/recipe_a")
    .arg("subdir2/recipe_b")
    .output()
    .unwrap();

  let stderr = String::from_utf8(output.stderr).unwrap();

  assert_eq!(
    stderr,
    "error: Conflicting path arguments: `subdir/` and `subdir2/`\n"
  );
}
