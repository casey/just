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
    .run();
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
    .status(EXIT_FAILURE)
    .stderr(
      "
      error: Could not find source file for import.
       ——▶ justfile:1:8
        │
      1 │ import './import.justfile'
        │        ^^^^^^^^^^^^^^^^^^^
      ",
    )
    .run();
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
    .run();
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
    .run();
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
    .run();
}

#[test]
fn circular_import() {
  Test::new()
    .justfile("import 'a'")
    .tree(tree! {
      a: "import 'b'",
      b: "import 'a'",
    })
    .status(EXIT_FAILURE)
    .stderr_regex(path_for_regex(
      "error: Import `.*/a` in `.*/b` is circular\n",
    ))
    .run();
}

#[test]
fn import_recipes_are_not_default() {
  Test::new()
    .tree(tree! {
      "import.justfile": "bar:",
    })
    .justfile("import './import.justfile'")
    .status(EXIT_FAILURE)
    .stderr("error: Justfile contains no default recipe.\n")
    .run();
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
    .run();
}

#[test]
fn include_error() {
  Test::new()
    .justfile("!include foo")
    .status(EXIT_FAILURE)
    .stderr(
      "
      error: The `!include` directive has been stabilized as `import`
       ——▶ justfile:1:1
        │
      1 │ !include foo
        │ ^
      ",
    )
    .run();
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
    .run();
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
    .run();
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
    .run();
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
    .run();
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
    .run();
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
    .run();
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
    .run();
}

#[test]
fn nested_import_paths_are_relative_to_containing_submodule() {
  Test::new()
    .justfile("import 'foo/import.just'")
    .write("foo/import.just", "import 'bar.just'")
    .write("foo/bar.just", "bar:\n @echo BAR")
    .arg("bar")
    .stdout("BAR\n")
    .run();
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
    .run();
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
    .run();
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
    .run();
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
    .run();
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
    .run();
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
    .run();
}

#[test]
fn glob_import_works() {
  Test::new()
    .tree(tree! {
      "a.just": "
        a:
          @echo A
      ",
      "b.just": "
        b:
          @echo B
      ",
    })
    .justfile(
      "
      set unstable

      import '*.just'

      default: a b
    ",
    )
    .stdout("A\nB\n")
    .test_round_trip(false)
    .run();
}

#[test]
fn glob_import_multiple_matches() {
  Test::new()
    .write(
      ".just/foo.justfile",
      "foo:\n  @echo FOO\n",
    )
    .write(
      ".just/bar.justfile",
      "bar:\n  @echo BAR\n",
    )
    .write(
      ".just/baz.justfile",
      "baz:\n  @echo BAZ\n",
    )
    .justfile(
      "
      set unstable

      import '.just/*.justfile'

      default: foo bar baz
    ",
    )
    .stdout("FOO\nBAR\nBAZ\n")
    .test_round_trip(false)
    .run();
}

#[test]
fn glob_import_optional_no_matches() {
  Test::new()
    .justfile(
      "
      set unstable

      import? '.just/*.justfile'

      default:
        @echo DONE
    ",
    )
    .stdout("DONE\n")
    .test_round_trip(false)
    .run();
}

#[test]
fn glob_import_required_no_matches() {
  Test::new()
    .justfile(
      "
      set unstable

      import '.just/*.justfile'

      default:
        @echo DONE
    ",
    )
    .status(EXIT_FAILURE)
    .stderr(
      "
      error: Could not find source file for import.
       ——▶ justfile:3:8
        │
      3 │ import '.just/*.justfile'
        │        ^^^^^^^^^^^^^^^^^^
    ",
    )
    .test_round_trip(false)
    .run();
}

#[test]
fn glob_import_without_unstable_feature() {
  Test::new()
    .tree(tree! {
      "a.just": "
        a:
          @echo A
      ",
    })
    .justfile(
      "
      import '*.just'

      default: a
    ",
    )
    .status(EXIT_FAILURE)
    .stderr("error: Glob patterns in imports are currently unstable. Invoke `just` with `--unstable`, set the `JUST_UNSTABLE` environment variable, or add `set unstable` to your `justfile` to enable unstable features.\n")
    .test_round_trip(false)
    .run();
}

#[test]
fn glob_import_deterministic_order() {
  Test::new()
    .tree(tree! {
      "c.just": "c := 'c'",
      "a.just": "a := 'a'",
      "b.just": "b := 'b'",
    })
    .justfile(
      "
      set unstable

      import '*.just'

      default:
        @echo {{a}}{{b}}{{c}}
    ",
    )
    .stdout("abc\n")
    .test_round_trip(false)
    .run();
}

#[test]
fn glob_import_circular_detection() {
  Test::new()
    .tree(tree! {
      "a.just": "import 'b.just'",
      "b.just": "import '*.just'",
    })
    .justfile(
      "
      set unstable

      import '*.just'
    ",
    )
    .status(EXIT_FAILURE)
    .stderr_regex(path_for_regex("error: Import `.*\\.just` in `.*\\.just` is circular\n"))
    .test_round_trip(false)
    .run();
}

#[test]
fn glob_import_with_prefix() {
  Test::new()
    .tree(tree! {
      "test-a.just": "
        a:
          @echo A
      ",
      "test-b.just": "
        b:
          @echo B
      ",
      "other.just": "
        c:
          @echo C
      ",
    })
    .justfile(
      "
      set unstable

      import 'test-*.just'

      default: a b
    ",
    )
    .stdout("A\nB\n")
    .test_round_trip(false)
    .run();
}

#[test]
fn glob_import_with_suffix() {
  Test::new()
    .tree(tree! {
      "a-test.just": "
        a:
          @echo A
      ",
      "b-test.just": "
        b:
          @echo B
      ",
      "other.just": "
        c:
          @echo C
      ",
    })
    .justfile(
      "
      set unstable

      import '*-test.just'

      default: a b
    ",
    )
    .stdout("A\nB\n")
    .test_round_trip(false)
    .run();
}

#[test]
fn glob_import_only_regular_files() {
  Test::new()
    .write(
      "a.just",
      "a:\n  @echo A\n",
    )
    .write("subdir/b.just", "")
    .justfile(
      "
      set unstable

      import '*.just'

      default: a
    ",
    )
    .stdout("A\n")
    .test_round_trip(false)
    .run();
}
