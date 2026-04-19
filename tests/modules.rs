use super::*;

#[test]
fn modules_are_stable() {
  Test::new()
    .justfile(
      "
        mod foo
      ",
    )
    .write("foo.just", "@bar:\n echo ok")
    .args(["foo", "bar"])
    .stdout("ok\n")
    .success();
}

#[test]
fn default_recipe_in_submodule_must_have_no_arguments() {
  Test::new()
    .write("foo.just", "foo bar:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo")
    .stderr("error: Recipe `foo` cannot be used as default recipe since it requires at least 1 argument.\n")
    .failure();
}

#[test]
fn module_recipes_can_be_run_as_subcommands() {
  Test::new()
    .write("foo.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo")
    .arg("foo")
    .stdout("FOO\n")
    .success();
}

#[test]
fn module_recipes_can_be_run_with_path_syntax() {
  Test::new()
    .write("foo.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo::foo")
    .stdout("FOO\n")
    .success();
}

#[test]
fn nested_module_recipes_can_be_run_with_path_syntax() {
  Test::new()
    .write("foo.just", "mod bar")
    .write("bar.just", "baz:\n @echo BAZ")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo::bar::baz")
    .stdout("BAZ\n")
    .success();
}

#[test]
fn invalid_path_syntax() {
  Test::new()
    .arg(":foo::foo")
    .stderr("error: Justfile does not contain recipe `:foo::foo`\n")
    .failure();

  Test::new()
    .arg("foo::foo:")
    .stderr("error: Justfile does not contain recipe `foo::foo:`\n")
    .failure();

  Test::new()
    .arg("foo:::foo")
    .stderr("error: Justfile does not contain recipe `foo:::foo`\n")
    .failure();
}

#[test]
fn missing_recipe_after_invalid_path() {
  Test::new()
    .arg(":foo::foo")
    .arg("bar")
    .stderr("error: Justfile does not contain recipe `:foo::foo`\n")
    .failure();
}

#[test]
fn assignments_are_evaluated_in_modules() {
  Test::new()
    .write("foo.just", "bar := 'CHILD'\nfoo:\n @echo {{bar}}")
    .justfile(
      "
        mod foo
        bar := 'PARENT'
      ",
    )
    .arg("foo")
    .arg("foo")
    .stdout("CHILD\n")
    .success();
}

#[test]
fn module_subcommand_runs_default_recipe() {
  Test::new()
    .write("foo.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo")
    .stdout("FOO\n")
    .success();
}

#[test]
fn modules_can_contain_other_modules() {
  Test::new()
    .write("bar.just", "baz:\n @echo BAZ")
    .write("foo.just", "mod bar")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo")
    .arg("bar")
    .arg("baz")
    .stdout("BAZ\n")
    .success();
}

#[test]
fn circular_module_imports_are_detected() {
  Test::new()
    .write("bar.just", "mod foo")
    .write("foo.just", "mod bar")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo")
    .arg("bar")
    .arg("baz")
    .stderr_regex(path_for_regex(
      "error: Import `.*/foo.just` in `.*/bar.just` is circular\n",
    ))
    .failure();
}

#[test]
fn modules_use_module_settings() {
  Test::new()
    .write(
      "foo.just",
      "set allow-duplicate-recipes
foo:
foo:
  @echo FOO
",
    )
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo")
    .arg("foo")
    .stdout("FOO\n")
    .success();

  Test::new()
    .write(
      "foo.just",
      "foo:
foo:
  @echo FOO
",
    )
    .justfile(
      "
        mod foo

        set allow-duplicate-recipes
      ",
    )
    .arg("foo")
    .arg("foo")
    .stderr(
      "
        error: Recipe `foo` first defined on line 1 is redefined on line 2
         ——▶ foo.just:2:1
          │
        2 │ foo:
          │ ^^^
      ",
    )
    .failure();
}

#[test]
fn modules_do_not_inherit_no_cd_setting() {
  Test::new()
    .write(
      "foo.just",
      "bar:
  @cat data.txt
",
    )
    .write("data.txt", "MODULE\n")
    .justfile(
      "
        set no-cd := true

        mod foo
      ",
    )
    .args(["foo", "bar"])
    .stdout("MODULE\n")
    .success();
}

#[test]
fn modules_conflict_with_recipes() {
  Test::new()
    .write("foo.just", "")
    .justfile(
      "
        mod foo
        foo:
      ",
    )
    .stderr(
      "
        error: Module `foo` defined on line 1 is redefined as a recipe on line 2
         ——▶ justfile:2:1
          │
        2 │ foo:
          │ ^^^
      ",
    )
    .failure();
}

#[test]
fn modules_conflict_with_aliases() {
  Test::new()
    .write("foo.just", "")
    .justfile(
      "
        mod foo
        bar:
        alias foo := bar
      ",
    )
    .stderr(
      "
        error: Module `foo` defined on line 1 is redefined as an alias on line 3
         ——▶ justfile:3:7
          │
        3 │ alias foo := bar
          │       ^^^
      ",
    )
    .failure();
}

#[test]
fn modules_conflict_with_other_modules() {
  Test::new()
    .write("foo.just", "")
    .justfile(
      "
        mod foo
        mod foo

        bar:
      ",
    )
    .stderr(
      "
        error: Module `foo` first defined on line 1 is redefined on line 2
         ——▶ justfile:2:5
          │
        2 │ mod foo
          │     ^^^
      ",
    )
    .failure();
}

#[test]
fn modules_are_dumped_correctly() {
  Test::new()
    .write("foo.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("--dump")
    .stdout("mod foo\n")
    .success();
}

#[test]
fn optional_modules_are_dumped_correctly() {
  Test::new()
    .write("foo.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod? foo
      ",
    )
    .arg("--dump")
    .stdout("mod? foo\n")
    .success();
}

#[test]
fn modules_can_be_in_subdirectory() {
  Test::new()
    .write("foo/mod.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo")
    .arg("foo")
    .stdout("FOO\n")
    .success();
}

#[test]
fn modules_in_subdirectory_can_be_named_justfile() {
  Test::new()
    .write("foo/justfile", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo")
    .arg("foo")
    .stdout("FOO\n")
    .success();
}

#[test]
fn modules_in_subdirectory_can_be_named_justfile_with_any_case() {
  Test::new()
    .write("foo/JUSTFILE", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo")
    .arg("foo")
    .stdout("FOO\n")
    .success();
}

#[test]
fn modules_in_subdirectory_can_have_leading_dot() {
  Test::new()
    .write("foo/.justfile", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo")
    .arg("foo")
    .stdout("FOO\n")
    .success();
}

#[test]
fn modules_require_unambiguous_file() {
  Test::new()
    .write("foo/justfile", "foo:\n @echo FOO")
    .write("foo.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .stderr(
      "
        error: Found multiple source files for module `foo`: `foo/justfile` and `foo.just`
         ——▶ justfile:1:5
          │
        1 │ mod foo
          │     ^^^
      "
      .replace('/', MAIN_SEPARATOR_STR),
    )
    .failure();
}

#[test]
fn missing_module_file_error() {
  Test::new()
    .justfile(
      "
        mod foo
      ",
    )
    .stderr(
      "
        error: Could not find source file for module `foo`.
         ——▶ justfile:1:5
          │
        1 │ mod foo
          │     ^^^
      ",
    )
    .failure();
}

#[test]
fn missing_optional_modules_do_not_trigger_error() {
  Test::new()
    .justfile(
      "
        mod? foo

        bar:
          @echo BAR
      ",
    )
    .stdout("BAR\n")
    .success();
}

#[test]
fn missing_optional_modules_do_not_conflict() {
  Test::new()
    .justfile(
      "
        mod? foo
        mod? foo
        mod foo 'baz.just'
      ",
    )
    .write("baz.just", "baz:\n @echo BAZ")
    .arg("foo")
    .arg("baz")
    .stdout("BAZ\n")
    .success();
}

#[test]
fn root_dotenv_is_available_to_submodules() {
  Test::new()
    .justfile(
      "
        set dotenv-load

        mod foo
      ",
    )
    .write("foo.just", "foo:\n @echo $DOTENV_KEY")
    .write(".env", "DOTENV_KEY=dotenv-value")
    .args(["foo", "foo"])
    .stdout("dotenv-value\n")
    .success();
}

#[test]
fn submodule_inherits_root_dotenv_even_with_dotenv_load_false() {
  Test::new()
    .justfile(
      "
        set dotenv-load

        mod foo
      ",
    )
    .write(
      "foo.just",
      "set dotenv-load := false\nfoo:\n @echo $DOTENV_KEY",
    )
    .write(".env", "DOTENV_KEY=dotenv-value")
    .args(["foo", "foo"])
    .stdout("dotenv-value\n")
    .success();
}

#[test]
fn submodule_loads_own_dotenv() {
  Test::new()
    .justfile(
      "
        set dotenv-load

        mod foo
      ",
    )
    .write(
      "foo/mod.just",
      "set dotenv-load\nfoo:\n @echo $ROOT_KEY $SUB_KEY",
    )
    .write(".env", "ROOT_KEY=root")
    .write("foo/.env", "SUB_KEY=sub")
    .args(["foo", "foo"])
    .stdout("root sub\n")
    .success();
}

#[test]
fn submodule_dotenv_overrides_root() {
  Test::new()
    .justfile(
      "
        set dotenv-load

        mod foo
      ",
    )
    .write("foo/mod.just", "set dotenv-load\nfoo:\n @echo $KEY")
    .write(".env", "KEY=root")
    .write("foo/.env", "KEY=sub")
    .args(["foo", "foo"])
    .stdout("sub\n")
    .success();
}

#[test]
fn root_does_not_see_submodule_dotenv() {
  Test::new()
    .justfile(
      "
        set dotenv-load

        mod foo

        bar:
          @echo ${SUB_KEY:-unset}
      ",
    )
    .write("foo/mod.just", "set dotenv-load\nfoo:\n @echo $SUB_KEY")
    .write(".env", "ROOT_KEY=root")
    .write("foo/.env", "SUB_KEY=sub")
    .args(["bar"])
    .stdout("unset\n")
    .success();
}

#[test]
fn nested_submodule_dotenv_merge() {
  Test::new()
    .justfile(
      "
        set dotenv-load

        mod foo
      ",
    )
    .write("foo/mod.just", "set dotenv-load\nmod bar")
    .write("foo/bar/mod.just", "set dotenv-load\nbaz:\n @echo $A $B $C")
    .write(".env", "A=1\nB=2\nC=3")
    .write("foo/.env", "B=20")
    .write("foo/bar/.env", "C=300")
    .args(["foo", "bar", "baz"])
    .stdout("1 20 300\n")
    .success();
}

#[test]
fn submodule_dotenv_load_false_skips_own_but_inherits_parent() {
  Test::new()
    .justfile(
      "
        set dotenv-load

        mod foo
      ",
    )
    .write(
      "foo/mod.just",
      "set dotenv-load := false\nfoo:\n @echo $ROOT_KEY ${SUB_KEY:-unset}",
    )
    .write(".env", "ROOT_KEY=root")
    .write("foo/.env", "SUB_KEY=sub")
    .args(["foo", "foo"])
    .stdout("root unset\n")
    .success();
}

#[test]
fn modules_may_specify_path() {
  Test::new()
    .write("commands/foo.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo 'commands/foo.just'
      ",
    )
    .arg("foo")
    .arg("foo")
    .stdout("FOO\n")
    .success();
}

#[test]
fn modules_may_specify_path_to_directory() {
  Test::new()
    .write("commands/bar/mod.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo 'commands/bar'
      ",
    )
    .arg("foo")
    .arg("foo")
    .stdout("FOO\n")
    .success();
}

#[test]
fn modules_with_paths_are_dumped_correctly() {
  Test::new()
    .write("commands/foo.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo 'commands/foo.just'
      ",
    )
    .arg("--dump")
    .stdout("mod foo 'commands/foo.just'\n")
    .success();
}

#[test]
fn optional_modules_with_paths_are_dumped_correctly() {
  Test::new()
    .write("commands/foo.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod? foo 'commands/foo.just'
      ",
    )
    .arg("--dump")
    .stdout("mod? foo 'commands/foo.just'\n")
    .success();
}

#[test]
fn recipes_may_be_named_mod() {
  Test::new()
    .justfile(
      "
        mod foo:
          @echo FOO
      ",
    )
    .arg("mod")
    .arg("bar")
    .stdout("FOO\n")
    .success();
}

#[test]
fn submodule_linewise_recipes_run_in_submodule_directory() {
  Test::new()
    .write("foo/bar", "BAR")
    .write("foo/mod.just", "foo:\n @cat bar")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo")
    .arg("foo")
    .stdout("BAR")
    .success();
}

#[test]
fn submodule_shebang_recipes_run_in_submodule_directory() {
  Test::new()
    .write("foo/bar", "BAR")
    .write("foo/mod.just", "foo:\n #!/bin/sh\n cat bar")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo")
    .arg("foo")
    .stdout("BAR")
    .success();
}

#[test]
fn cross_module_dependency_runs_in_submodule_directory() {
  Test::new()
    .write("foo/bar", "BAR")
    .write("foo/mod.just", "foo:\n @cat bar")
    .justfile(
      "
        mod foo

        main: foo::foo
      ",
    )
    .arg("main")
    .stdout("BAR")
    .success();
}

#[test]
fn cross_module_dependency_with_no_cd_runs_in_invocation_directory() {
  Test::new()
    .write("root_file", "ROOT")
    .write(
      "foo/mod.just",
      "
[no-cd]
foo:
  @cat root_file
      ",
    )
    .justfile(
      "
        mod foo

        main: foo::foo
      ",
    )
    .arg("main")
    .stdout("ROOT")
    .success();
}

#[test]
fn nested_cross_module_dependency_runs_in_correct_directory() {
  Test::new()
    .write("outer/inner/file", "NESTED")
    .write("outer/inner/mod.just", "task:\n @cat file")
    .write("outer/mod.just", "mod inner")
    .justfile(
      "
        mod outer

        main: outer::inner::task
      ",
    )
    .arg("main")
    .stdout("NESTED")
    .success();
}

#[test]
fn modulepaths_beginning_with_tilde_are_expanded_to_homdir() {
  if cfg!(windows) {
    return;
  }
  Test::new()
    .write("foobar/mod.just", "foo:\n @echo FOOBAR")
    .justfile(
      "
        mod foo '~/mod.just'
      ",
    )
    .arg("foo")
    .arg("foo")
    .stdout("FOOBAR\n")
    .env("HOME", "foobar")
    .success();
}

#[test]
fn recipes_with_same_name_are_both_run() {
  Test::new()
    .write("foo.just", "bar:\n @echo MODULE")
    .justfile(
      "
        mod foo

        bar:
          @echo ROOT
      ",
    )
    .arg("foo::bar")
    .arg("bar")
    .stdout("MODULE\nROOT\n")
    .success();
}

#[test]
fn submodule_recipe_not_found_error_message() {
  Test::new()
    .args(["foo::bar"])
    .stderr("error: Justfile does not contain submodule `foo`\n")
    .failure();
}

#[test]
fn submodule_recipe_not_found_spaced_error_message() {
  Test::new()
    .write("foo.just", "bar:\n @echo MODULE")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["foo", "baz"])
    .stderr("error: Justfile does not contain recipe `foo baz`\nDid you mean `bar`?\n")
    .failure();
}

#[test]
fn submodule_recipe_not_found_colon_separated_error_message() {
  Test::new()
    .write("foo.just", "bar:\n @echo MODULE")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["foo::baz"])
    .stderr("error: Justfile does not contain recipe `foo::baz`\nDid you mean `bar`?\n")
    .failure();
}

#[test]
fn colon_separated_path_does_not_run_recipes() {
  Test::new()
    .justfile(
      "
        foo:
          @echo FOO

        bar:
          @echo BAR
      ",
    )
    .args(["foo::bar"])
    .stderr("error: Expected submodule at `foo` but found recipe.\n")
    .failure();
}

#[test]
fn expected_submodule_but_found_recipe_in_root_error() {
  Test::new()
    .justfile("foo:")
    .arg("foo::baz")
    .stderr("error: Expected submodule at `foo` but found recipe.\n")
    .failure();
}

#[test]
fn expected_submodule_but_found_recipe_in_submodule_error() {
  Test::new()
    .justfile("mod foo")
    .write("foo.just", "bar:")
    .args(["foo::bar::baz"])
    .stderr("error: Expected submodule at `foo::bar` but found recipe.\n")
    .failure();
}

#[test]
fn colon_separated_path_components_are_not_used_as_arguments() {
  Test::new()
    .justfile("foo bar:")
    .args(["foo::bar"])
    .stderr("error: Expected submodule at `foo` but found recipe.\n")
    .failure();
}

#[test]
fn comments_can_follow_modules() {
  Test::new()
    .write("foo.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo # this is foo
      ",
    )
    .args(["foo", "foo"])
    .stdout("FOO\n")
    .success();
}

#[test]
fn doc_comment_on_module() {
  Test::new()
    .write("foo.just", "")
    .justfile(
      "
        # Comment
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--list")
    .stdout("Available recipes:\n    foo ... # Comment\n")
    .success();
}

#[test]
fn doc_attribute_on_module() {
  Test::new()
    .write("foo.just", "")
    .justfile(
      "
        # Suppressed comment
        [doc: 'Comment']
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--list")
    .stdout("Available recipes:\n    foo ... # Comment\n")
    .success();
}

#[test]
fn group_attribute_on_module() {
  Test::new()
    .write("foo.just", "")
    .write("bar.just", "")
    .write("zee.just", "")
    .justfile(
      "
        [group('alpha')]
        mod zee

        [group('alpha')]
        mod foo

        [group('alpha')]
        a:

        [group('beta')]
        b:

        [group('beta')]
        mod bar

        c:
      ",
    )
    .test_round_trip(false)
    .arg("--list")
    .stdout(
      "
        Available recipes:
            c

            [alpha]
            a
            foo ...
            zee ...

            [beta]
            b
            bar ...
      ",
    )
    .success();
}

#[test]
fn group_attribute_on_module_unsorted() {
  Test::new()
    .write("foo.just", "")
    .write("bar.just", "")
    .write("zee.just", "")
    .justfile(
      "
        [group('alpha')]
        mod zee

        [group('alpha')]
        mod foo

        [group('alpha')]
        a:

        [group('beta')]
        b:

        [group('beta')]
        mod bar

        c:
      ",
    )
    .test_round_trip(false)
    .arg("--list")
    .arg("--unsorted")
    .stdout(
      "
        Available recipes:
            c

            [alpha]
            a
            zee ...
            foo ...

            [beta]
            b
            bar ...
      ",
    )
    .success();
}

#[test]
fn group_attribute_on_module_list_submodule() {
  Test::new()
    .write("foo.just", "d:")
    .write("bar.just", "e:")
    .write("zee.just", "f:")
    .justfile(
      "
        [group('alpha')]
        mod zee

        [group('alpha')]
        mod foo

        [group('alpha')]
        a:

        [group('beta')]
        b:

        [group('beta')]
        mod bar

        c:
      ",
    )
    .test_round_trip(false)
    .arg("--list")
    .arg("--list-submodules")
    .stdout(
      "
        Available recipes:
            c

            [alpha]
            a
            foo:
                d
            zee:
                f

            [beta]
            b
            bar:
                e
      ",
    )
    .success();
}

#[test]
fn group_attribute_on_module_list_submodule_unsorted() {
  Test::new()
    .write("foo.just", "d:")
    .write("bar.just", "e:")
    .write("zee.just", "f:")
    .justfile(
      "
        [group('alpha')]
        mod zee

        [group('alpha')]
        mod foo

        [group('alpha')]
        a:

        [group('beta')]
        b:

        [group('beta')]
        mod bar

        c:
      ",
    )
    .test_round_trip(false)
    .arg("--list")
    .arg("--list-submodules")
    .arg("--unsorted")
    .stdout(
      "
        Available recipes:
            c

            [alpha]
            a
            zee:
                f
            foo:
                d

            [beta]
            b
            bar:
                e
      ",
    )
    .success();
}

#[test]
fn bad_module_attribute_fails() {
  Test::new()
    .write("foo.just", "")
    .justfile(
      "
        [no-cd]
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--list")
    .stderr("error: Module `foo` has invalid attribute `no-cd`\n ——▶ justfile:2:5\n  │\n2 │ mod foo\n  │     ^^^\n")
    .failure();
}

#[test]
fn empty_doc_attribute_on_module() {
  Test::new()
    .write("foo.just", "")
    .justfile(
      "
        # Suppressed comment
        [doc]
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--list")
    .stdout("Available recipes:\n    foo ...\n")
    .success();
}

#[test]
fn overrides_work_when_submodule_is_present() {
  Test::new()
    .write("bar.just", "")
    .justfile(
      "
        mod bar

        x := 'a'

        foo:
          @echo {{ x }}
      ",
    )
    .test_round_trip(false)
    .arg("x=b")
    .stdout("b\n")
    .success();
}

#[test]
fn exported_variables_are_available_in_submodules() {
  Test::new()
    .write("foo.just", "bar:\n @echo $x")
    .justfile(
      "
        mod foo

        export x := 'a'
      ",
    )
    .test_round_trip(false)
    .arg("foo::bar")
    .stdout("a\n")
    .success();
}

#[test]
fn exported_variables_can_be_unexported_in_submodules() {
  Test::new()
    .write("foo.just", "unexport x\nbar:\n @echo ${x:-default}")
    .justfile(
      "
        mod foo

        export x := 'a'
      ",
    )
    .test_round_trip(false)
    .arg("foo::bar")
    .stdout("default\n")
    .success();
}

#[test]
fn exported_variables_can_be_overridden_in_submodules() {
  Test::new()
    .write("foo.just", "export x := 'b'\nbar:\n @echo $x")
    .justfile(
      "
        mod foo

        export x := 'a'
      ",
    )
    .test_round_trip(false)
    .arg("foo::bar")
    .stdout("b\n")
    .success();
}

#[test]
fn verbose_message_includes_module_path() {
  Test::new()
    .write("foo.just", "bar:\n @echo BAR")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--verbose", "foo::bar"])
    .stderr("===> Running recipe `foo::bar`...\necho BAR\n")
    .stdout("BAR\n")
    .success();
}

#[test]
fn trailing_separator_runs_default_recipe() {
  Test::new()
    .write("foo.just", "@bar:\n echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo::")
    .stdout("FOO\n")
    .success();
}

#[test]
fn nested_trailing_separator_runs_default_recipe() {
  Test::new()
    .write("foo.just", "mod bar")
    .write("bar.just", "@baz:\n echo BAZ")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo::bar::")
    .stdout("BAZ\n")
    .success();
}

#[test]
fn trailing_separator_no_default_recipe() {
  Test::new()
    .write("foo.just", "import 'bar.just'")
    .write("bar.just", "bar:\n @echo BAR")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo::")
    .stderr("error: Justfile contains no default recipe.\n")
    .failure();
}

#[test]
fn trailing_separator_not_last_argument() {
  Test::new()
    .write("foo.just", "bar:\n @echo BAR")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["foo::", "bar"])
    .stderr("error: Justfile does not contain recipe `foo::`\n")
    .failure();
}
