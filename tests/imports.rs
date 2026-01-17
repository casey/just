use super::*;

#[test]
fn import_succeeds() {
  Test::new()
    .tree(tree! {
      "import.justfile": "
        b:
          @echo B
      ",
    })
    .justfile(
      "
        import './import.justfile'

        a: b
          @echo A
      ",
    )
    .arg("a")
    .stdout("B\nA\n")
    .success();
}

#[test]
fn missing_import_file_error() {
  Test::new()
    .justfile(
      "
        import './import.justfile'

        a:
          @echo A
      ",
    )
    .arg("a")
    .stderr(
      "
      error: Could not find source file for import.
       ——▶ justfile:1:8
        │
      1 │ import './import.justfile'
        │        ^^^^^^^^^^^^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn missing_optional_imports_are_ignored() {
  Test::new()
    .justfile(
      "
        import? './import.justfile'

        a:
          @echo A
      ",
    )
    .arg("a")
    .stdout("A\n")
    .success();
}

#[test]
fn trailing_spaces_after_import_are_ignored() {
  Test::new()
    .tree(tree! {
      "import.justfile": "",
    })
    .justfile(
      "
      import './import.justfile'\x20
      a:
        @echo A
    ",
    )
    .stdout("A\n")
    .success();
}

#[test]
fn import_after_recipe() {
  Test::new()
    .tree(tree! {
      "import.justfile": "
        a:
          @echo A
      ",
    })
    .justfile(
      "
      b: a
      import './import.justfile'
      ",
    )
    .stdout("A\n")
    .success();
}

#[test]
fn circular_import() {
  Test::new()
    .justfile("import 'a'")
    .tree(tree! {
      a: "import 'b'",
      b: "import 'a'",
    })
    .stderr_regex(path_for_regex(
      "error: Import `.*/a` in `.*/b` is circular\n",
    ))
    .failure();
}

#[test]
fn import_recipes_are_not_default() {
  Test::new()
    .tree(tree! {
      "import.justfile": "bar:",
    })
    .justfile("import './import.justfile'")
    .stderr("error: Justfile contains no default recipe.\n")
    .failure();
}

#[test]
fn listed_recipes_in_imports_are_in_load_order() {
  Test::new()
    .justfile(
      "
      import './import.justfile'
      foo:
    ",
    )
    .write("import.justfile", "bar:")
    .args(["--list", "--unsorted"])
    .stdout(
      "
      Available recipes:
          foo
          bar
    ",
    )
    .success();
}

#[test]
fn include_error() {
  Test::new()
    .justfile("!include foo")
    .stderr(
      "
      error: The `!include` directive has been stabilized as `import`
       ——▶ justfile:1:1
        │
      1 │ !include foo
        │ ^
      ",
    )
    .failure();
}

#[test]
fn recipes_in_import_are_overridden_by_recipes_in_parent() {
  Test::new()
    .tree(tree! {
      "import.justfile": "
        a:
          @echo IMPORT
      ",
    })
    .justfile(
      "
        a:
          @echo ROOT

        import './import.justfile'

        set allow-duplicate-recipes
      ",
    )
    .arg("a")
    .stdout("ROOT\n")
    .success();
}

#[test]
fn variables_in_import_are_overridden_by_variables_in_parent() {
  Test::new()
    .tree(tree! {
      "import.justfile": "
    f := 'foo'
    ",
    })
    .justfile(
      "
      f := 'bar'

      import './import.justfile'

      set allow-duplicate-variables

      a:
        @echo {{f}}
    ",
    )
    .arg("a")
    .stdout("bar\n")
    .success();
}

#[cfg(not(windows))]
#[test]
fn import_paths_beginning_with_tilde_are_expanded_to_homdir() {
  Test::new()
    .write("foobar/mod.just", "foo:\n @echo FOOBAR")
    .justfile(
      "
        import '~/mod.just'
      ",
    )
    .arg("foo")
    .stdout("FOOBAR\n")
    .env("HOME", "foobar")
    .success();
}

#[test]
fn imports_dump_correctly() {
  Test::new()
    .write("import.justfile", "")
    .justfile(
      "
        import './import.justfile'
      ",
    )
    .arg("--dump")
    .stdout("import './import.justfile'\n")
    .success();
}

#[test]
fn optional_imports_dump_correctly() {
  Test::new()
    .write("import.justfile", "")
    .justfile(
      "
        import? './import.justfile'
      ",
    )
    .arg("--dump")
    .stdout("import? './import.justfile'\n")
    .success();
}

#[test]
fn imports_in_root_run_in_justfile_directory() {
  Test::new()
    .write("foo/import.justfile", "bar:\n @cat baz")
    .write("baz", "BAZ")
    .justfile(
      "
        import 'foo/import.justfile'
      ",
    )
    .arg("bar")
    .stdout("BAZ")
    .success();
}

#[test]
fn imports_in_submodules_run_in_submodule_directory() {
  Test::new()
    .justfile("mod foo")
    .write("foo/mod.just", "import 'import.just'")
    .write("foo/import.just", "bar:\n @cat baz")
    .write("foo/baz", "BAZ")
    .arg("foo")
    .arg("bar")
    .stdout("BAZ")
    .success();
}

#[test]
fn nested_import_paths_are_relative_to_containing_submodule() {
  Test::new()
    .justfile("import 'foo/import.just'")
    .write("foo/import.just", "import 'bar.just'")
    .write("foo/bar.just", "bar:\n @echo BAR")
    .arg("bar")
    .stdout("BAR\n")
    .success();
}

#[test]
fn recipes_in_nested_imports_run_in_parent_module() {
  Test::new()
    .justfile("import 'foo/import.just'")
    .write("foo/import.just", "import 'bar/import.just'")
    .write("foo/bar/import.just", "bar:\n @cat baz")
    .write("baz", "BAZ")
    .arg("bar")
    .stdout("BAZ")
    .success();
}

#[test]
fn shebang_recipes_in_imports_in_root_run_in_justfile_directory() {
  Test::new()
    .write(
      "foo/import.justfile",
      "bar:\n #!/usr/bin/env bash\n cat baz",
    )
    .write("baz", "BAZ")
    .justfile(
      "
        import 'foo/import.justfile'
      ",
    )
    .arg("bar")
    .stdout("BAZ")
    .success();
}

#[test]
fn recipes_imported_in_root_run_in_command_line_provided_working_directory() {
  Test::new()
    .write("subdir/b.justfile", "@b:\n  cat baz")
    .write("subdir/a.justfile", "import 'b.justfile'\n@a: b\n  cat baz")
    .write("baz", "BAZ")
    .args([
      "--working-directory",
      ".",
      "--justfile",
      "subdir/a.justfile",
    ])
    .stdout("BAZBAZ")
    .success();
}

#[test]
fn reused_import_are_allowed() {
  Test::new()
    .justfile(
      "
      import 'a'
      import 'b'

      bar:
    ",
    )
    .tree(tree! {
      a: "import 'c'",
      b: "import 'c'",
      c: "",
    })
    .success();
}

#[test]
fn multiply_imported_items_do_not_conflict() {
  Test::new()
    .justfile(
      "
      import 'a.just'
      import 'a.just'
      foo: bar
    ",
    )
    .write(
      "a.just",
      "
x := 'y'

@bar:
  echo hello
",
    )
    .stdout("hello\n")
    .success();
}

#[test]
fn nested_multiply_imported_items_do_not_conflict() {
  Test::new()
    .justfile(
      "
      import 'a.just'
      import 'b.just'
      foo: bar
    ",
    )
    .write("a.just", "import 'c.just'")
    .write("b.just", "import 'c.just'")
    .write(
      "c.just",
      "
x := 'y'

@bar:
  echo hello
",
    )
    .stdout("hello\n")
    .success();
}
