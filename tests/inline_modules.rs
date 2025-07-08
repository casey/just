use super::*;

#[test]
fn inline_module() {
  Test::new()
    .justfile(
      r"
      foo::
        bar:
          @echo BAR",
    )
    .arg("foo::bar")
    .stdout("BAR\n")
    .run();
}

#[test]
fn inline_module_normal_recipe_works() {
  Test::new()
    .justfile(
      r"
      bar:
        @echo BAR

      foo::
        bar:
          @echo SHOULDNT_BE_HERE",
    )
    .arg("bar")
    .stdout("BAR\n")
    .run();
}

#[test]
fn inline_module_normal_recipe_works_module_first() {
  Test::new()
    .justfile(
      r"
      foo::
        bar:
          @echo SHOULDNT_BE_HERE

      bar:
        @echo BAR",
    )
    .arg("bar")
    .stdout("BAR\n")
    .run();
}

// Remove once nesting support is added
#[test]
fn double_inline_module_fails() {
  Test::new()
    .justfile(
      r"
      foo::
        bar::
          baz:
            @echo BAZ",
    )
    .arg("foo::bar::baz")
    .test_round_trip(false)
    .status(1)
    .stderr("error: Inline module `foo` cannot be nested\n ——▶ justfile:2:3\n  │\n2 │   bar::\n  │   ^^^\n")
    .run();
}

// Remove once nesting support is added
#[test]
fn module_in_inline_module_fails() {
  Test::new()
    .justfile(
      r"
      foo::
        mod bar
        ",
    )
    .arg("foo::bar::baz")
    .test_round_trip(false)
    .status(1)
    .stderr(
      "error: Inline module `foo` cannot be nested\n ——▶ justfile:2:7\n  │\n2 │   mod bar\n  │       ^^^\n",
    )
    .run();
}

#[test]
fn module_and_inline_module_of_same_name_fails() {
  Test::new()
    .write("foo.just", "")
    .justfile(
      r"
      mod foo

      foo::

        ",
    )
    .arg("foo::bar::baz")
    .test_round_trip(false)
    .status(1)
    .stderr("error: Module `foo` first defined on line 1 is redefined on line 3\n ——▶ justfile:3:1\n  │\n3 │ foo::\n  │ ^^^\n")
    .run();
}

#[test]
fn bad_inline_module_attribute_fails() {
  Test::new()
    .justfile(
      r"
        [no-cd]
        foo::
      ",
    )
    .test_round_trip(false)
    .arg("--list")
    .stderr("error: Module `foo` has invalid attribute `no-cd`\n ——▶ justfile:2:1\n  │\n2 │ foo::\n  │ ^^^\n")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn empty_doc_attribute_on_inline_module() {
  Test::new()
    .justfile(
      r"
        # Suppressed comment
        [doc]
        foo::
      ",
    )
    .test_round_trip(false)
    .arg("--list")
    .stdout("Available recipes:\n    foo ...\n")
    .run();
}
