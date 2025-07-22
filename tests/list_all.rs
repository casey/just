use super::*;

#[test]
fn list_all_basic() {
  Test::new()
    .write("foo.just", "bar:\n @echo BAR")
    .justfile(
      "
        recipe:
          @echo RECIPE

        mod foo
      ",
    )
    .args(["--list-all"])
    .stdout(
      "
        Available recipes:
            recipe
            foo:
                bar
      ",
    )
    .run();
}

#[test]
fn list_all_nested_modules() {
  Test::new()
    .write("foo.just", "mod bar")
    .write("bar.just", "baz:\n @echo BAZ")
    .justfile(
      "
        top:
          @echo TOP

        mod foo
      ",
    )
    .args(["--list-all"])
    .stdout(
      "
        Available recipes:
            top
            foo:
                bar:
                    baz
      ",
    )
    .run();
}

#[test]
fn list_all_with_groups() {
  Test::new()
    .justfile(
      "
        [group('ROOT')]
        root:

        mod foo
      ",
    )
    .write("foo.just", "[group('FOO')]\nfoo_recipe:")
    .args(["--list-all"])
    .stdout(
      "
        Available recipes:
            foo:
                [FOO]
                foo_recipe

            [ROOT]
            root
      ",
    )
    .run();
}

#[test]
fn list_all_module_path() {
  Test::new()
    .write("foo.just", "mod bar")
    .write("bar.just", "baz:\n @echo BAZ\n\nqux:\n @echo QUX")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--list-all", "foo"])
    .stdout(
      "
        Available recipes:
            bar:
                baz
                qux
      ",
    )
    .run();
}

#[test]
fn list_all_nested_module_path() {
  Test::new()
    .write("foo.just", "mod bar")
    .write("bar.just", "baz:\n @echo BAZ")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--list-all", "foo", "bar"])
    .stdout(
      "
        Available recipes:
            baz
      ",
    )
    .run();

  Test::new()
    .write("foo.just", "mod bar")
    .write("bar.just", "baz:\n @echo BAZ")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--list-all", "foo::bar"])
    .stdout(
      "
        Available recipes:
            baz
      ",
    )
    .run();
}

#[test]
fn list_all_with_comments() {
  Test::new()
    .write("foo.just", "# Foo recipe\nbar:\n @echo BAR")
    .justfile(
      "
        # Root recipe
        recipe:
          @echo RECIPE

        mod foo
      ",
    )
    .args(["--list-all"])
    .stdout(
      "
        Available recipes:
            recipe # Root recipe
            foo:
                bar # Foo recipe
      ",
    )
    .run();
}

#[test]
fn list_all_ignores_private_recipes() {
  Test::new()
    .write(
      "foo.just",
      "
public:
  @echo PUBLIC

_private:
  @echo PRIVATE

[private]
also_private:
  @echo ALSO_PRIVATE
      ",
    )
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--list-all"])
    .stdout(
      "
        Available recipes:
            foo:
                public
      ",
    )
    .run();
}

#[test]
fn list_all_multiple_modules() {
  Test::new()
    .write("alpha.just", "alpha_recipe:")
    .write("beta.just", "beta_recipe:")
    .write("gamma.just", "gamma_recipe:")
    .justfile(
      "
        mod alpha
        mod beta
        mod gamma
      ",
    )
    .args(["--list-all"])
    .stdout(
      "
        Available recipes:
            alpha:
                alpha_recipe

            beta:
                beta_recipe

            gamma:
                gamma_recipe
      ",
    )
    .run();
}

#[test]
fn list_all_with_module_doc() {
  Test::new()
    .write("foo.just", "bar:")
    .justfile(
      "
        # Module documentation
        mod foo
      ",
    )
    .args(["--list-all"])
    .stdout(
      "
        Available recipes:
            foo: # Module documentation
                bar
      ",
    )
    .run();
}

#[test]
fn list_all_complex_hierarchy() {
  Test::new()
    .write("a.just", "mod b\nmod c\n\na_recipe:")
    .write("b.just", "b_recipe:")
    .write("c.just", "mod d\n\nc_recipe:")
    .write("d.just", "d_recipe:")
    .justfile(
      "
        root:

        mod a
      ",
    )
    .args(["--list-all"])
    .stdout(
      "
        Available recipes:
            root
            a:
                a_recipe
                b:
                    b_recipe
                c:
                    c_recipe
                    d:
                        d_recipe
      ",
    )
    .run();
}

#[test]
fn list_all_unsorted() {
  Test::new()
    .write("foo.just", "zzz:\n\naaa:")
    .write("bar.just", "bbb:")
    .justfile(
      "
        yyy:
        xxx:

        mod foo
        mod bar
      ",
    )
    .args(["--list-all", "--unsorted"])
    .stdout(
      "
        Available recipes:
            yyy
            xxx
            foo:
                zzz
                aaa

            bar:
                bbb
      ",
    )
    .run();
}

#[test]
fn list_all_with_aliases() {
  Test::new()
    .write("foo.just", "bar:\n\nalias b := bar")
    .justfile(
      "
        recipe:

        alias r := recipe

        mod foo
      ",
    )
    .args(["--list-all"])
    .stdout(
      "
        Available recipes:
            recipe
            r      # alias for `recipe`
            foo:
                bar
                b   # alias for `bar`
      ",
    )
    .run();
}

#[test]
fn list_all_invalid_path() {
  Test::new()
    .args(["--list-all", "$invalid"])
    .stderr("error: Invalid module path `$invalid`\n")
    .status(1)
    .run();
}

#[test]
fn list_all_unknown_submodule() {
  Test::new()
    .args(["--list-all", "unknown"])
    .stderr("error: Justfile does not contain submodule `unknown`\n")
    .status(1)
    .run();
}

#[test]
fn list_all_conflicts_with_list() {
  Test::new()
    .args(["--list", "--list-all"])
    .stderr_regex("error: the argument '--list \\[<MODULE>...\\]' cannot be used with '--list-all \\[<MODULE>...\\]'.*")
    .status(2)
    .run();
}

#[test]
fn list_all_with_imports() {
  Test::new()
    .write("imported.just", "imported_recipe:")
    .write("module.just", "module_recipe:")
    .justfile(
      "
        import 'imported.just'
        
        mod module
        
        main_recipe:
      ",
    )
    .args(["--list-all"])
    .stdout(
      "
        Available recipes:
            imported_recipe
            main_recipe
            module:
                module_recipe
      ",
    )
    .run();
}

#[test]
fn list_all_backticks_highlighted() {
  Test::new()
    .write("foo.just", "# Module recipe `with backticks`\nbar:")
    .justfile(
      "
        # Root `with backticks` too
        recipe:

        mod foo
      ",
    )
    .args(["--list-all", "--color=always"])
    .stdout(
      "
        Available recipes:
            recipe \u{1b}[34m#\u{1b}[0m \u{1b}[34mRoot \u{1b}[0m\u{1b}[36m`with backticks`\u{1b}[0m\u{1b}[34m too\u{1b}[0m
            foo:
                bar \u{1b}[34m#\u{1b}[0m \u{1b}[34mModule recipe \u{1b}[0m\u{1b}[36m`with backticks`\u{1b}[0m
      ",
    )
    .run();
}

#[test]
fn list_all_empty_modules() {
  Test::new()
    .write("empty.just", "")
    .justfile(
      "
        recipe:

        mod empty
      ",
    )
    .args(["--list-all"])
    .stdout(
      "
        Available recipes:
            recipe
            empty:
      ",
    )
    .run();
}

#[test]
fn list_all_module_with_space_group() {
  Test::new()
    .write("foo.just", "[group(' ')]\nbar:")
    .write("baz.just", "qux:")
    .justfile(
      "
        mod foo
        mod baz
      ",
    )
    .args(["--list-all"])
    .stdout(
      "
        Available recipes:
            baz:
                qux

            foo:
                [ ]
                bar
      ",
    )
    .run();
}