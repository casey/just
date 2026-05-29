use super::*;

#[test]
fn default_attribute_overrides_first_recipe() {
  Test::new()
    .justfile(
      "
        foo:
          @echo FOO

        [default]
        bar:
          @echo BAR
      ",
    )
    .stdout("BAR\n")
    .success();
}

#[test]
fn default_attribute_may_only_appear_once_per_justfile() {
  Test::new()
    .justfile(
      "
        [default]
        foo:

        [default]
        bar:
      ",
    )
    .stderr(
      "
        error: recipe `foo` has duplicate `[default]` attribute, which may only appear once per module
         ——▶ justfile:2:1
          │
        2 │ foo:
          │ ^^^
      "
    )
    .failure();
}

#[test]
fn default_list_lists_recipes() {
  Test::new()
    .justfile(
      "
        set default-list := true

        foo:
          @echo foo

        bar:
      ",
    )
    .stdout(
      "
        Available recipes:
            bar
            foo
      ",
    )
    .success();
}

#[test]
fn default_list_flag_lists_recipes() {
  Test::new()
    .justfile(
      "
        foo:
          @echo foo

        bar:
      ",
    )
    .arg("--default-list")
    .stdout(
      "
        Available recipes:
            bar
            foo
      ",
    )
    .success();
}

#[test]
fn default_list_flag_does_not_override_explicit_recipe() {
  Test::new()
    .justfile(
      "
        foo:
          @echo foo

        bar:
          @echo bar
      ",
    )
    .arg("--default-list")
    .arg("bar")
    .stdout("bar\n")
    .success();
}

#[test]
fn default_list_false_runs_default_recipe() {
  Test::new()
    .justfile(
      "
        set default-list := false

        foo:
          @echo foo

        bar:
      ",
    )
    .stdout("foo\n")
    .success();
}

#[test]
fn default_list_does_not_override_explicit_recipe() {
  Test::new()
    .justfile(
      "
        set default-list

        foo:
          @echo foo

        bar:
          @echo bar
      ",
    )
    .arg("bar")
    .stdout("bar\n")
    .success();
}

#[test]
fn default_list_allows_default_recipe_with_arguments() {
  Test::new()
    .justfile(
      "
        set default-list

        foo bar:
          @echo {{bar}}

        baz:
      ",
    )
    .stdout(
      "
        Available recipes:
            baz
            foo bar
      ",
    )
    .success();
}
