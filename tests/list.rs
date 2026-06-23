use super::*;

#[test]
fn list_recipes_in_search_directory() {
  Test::new()
    .justfile("root-recipe:")
    .write("child/justfile", "child-recipe:")
    .current_dir("child")
    .args(["--list", ".."])
    .stdout(
      "
        Available recipes:
            root-recipe
      ",
    )
    .success();
}

#[test]
fn modules_unsorted() {
  Test::new()
    .write("foo.just", "foo:")
    .write("bar.just", "bar:")
    .justfile(
      "
        mod foo

        mod bar
      ",
    )
    .args(["--list", "--unsorted"])
    .stdout(
      "
        Available recipes:
            foo ...
            bar ...
      ",
    )
    .success();
}

#[test]
fn unsorted_list_order() {
  Test::new()
    .write("a.just", "a:")
    .write("b.just", "b:")
    .write("c.just", "c:")
    .write("d.just", "d:")
    .justfile(
      "
        import 'a.just'
        import 'b.just'
        import 'c.just'
        import 'd.just'
        x:
        y:
        z:
      ",
    )
    .args(["--list", "--unsorted"])
    .stdout(
      "
        Available recipes:
            x
            y
            z
            a
            b
            c
            d
      ",
    )
    .success();

  Test::new()
    .write("a.just", "a:")
    .write("b.just", "b:")
    .write("c.just", "c:")
    .write("d.just", "d:")
    .justfile(
      "
        x:
        y:
        z:
        import 'd.just'
        import 'c.just'
        import 'b.just'
        import 'a.just'
      ",
    )
    .args(["--list", "--unsorted"])
    .stdout(
      "
        Available recipes:
            x
            y
            z
            d
            c
            b
            a
      ",
    )
    .success();

  Test::new()
    .write(
      "a.just",
      "
        a:
        import 'e.just'
      ",
    )
    .write(
      "b.just",
      "
        b:
        import 'f.just'
      ",
    )
    .write(
      "c.just",
      "
        c:
        import 'g.just'
      ",
    )
    .write(
      "d.just",
      "
        d:
        import 'h.just'
      ",
    )
    .write("e.just", "e:")
    .write("f.just", "f:")
    .write("g.just", "g:")
    .write("h.just", "h:")
    .justfile(
      "
        x:
        y:
        z:
        import 'd.just'
        import 'c.just'
        import 'b.just'
        import 'a.just'
      ",
    )
    .args(["--list", "--unsorted"])
    .stdout(
      "
        Available recipes:
            x
            y
            z
            d
            h
            c
            g
            b
            f
            a
            e
      ",
    )
    .success();

  Test::new()
    .write("task1.just", "task1:")
    .write("task2.just", "task2:")
    .justfile(
      "
        import 'task1.just'
        import 'task2.just'
      ",
    )
    .args(["--list", "--unsorted"])
    .stdout(
      "
        Available recipes:
            task1
            task2
      ",
    )
    .success();
}

#[test]
fn list_submodule() {
  Test::new()
    .write("foo.just", "bar:")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--list", "foo"])
    .stdout(
      "
        Available recipes:
            bar
      ",
    )
    .success();
}

#[test]
fn list_nested_submodule() {
  Test::new()
    .write("foo.just", "mod bar")
    .write("bar.just", "baz:")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--list", "foo", "bar"])
    .stdout(
      "
        Available recipes:
            baz
      ",
    )
    .success();

  Test::new()
    .write("foo.just", "mod bar")
    .write("bar.just", "baz:")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--list", "foo::bar"])
    .stdout(
      "
        Available recipes:
            baz
      ",
    )
    .success();
}

#[test]
fn list_invalid_path() {
  Test::new()
    .args(["--list", "$hello"])
    .stderr("error: invalid module path `$hello`\n")
    .failure();
}

#[test]
fn list_unknown_submodule() {
  Test::new()
    .justfile("")
    .args(["--list", "hello"])
    .stderr("error: justfile does not contain submodule `hello`\n")
    .failure();
}

#[test]
fn list_with_groups_in_modules() {
  Test::new()
    .justfile(
      "
        [group('FOO')]
        foo:

        mod bar
      ",
    )
    .write(
      "bar.just",
      "
        [group('BAZ')]
        baz:
      ",
    )
    .args(["--list", "--list-submodules"])
    .stdout(
      "
        Available recipes:
            bar:
                [BAZ]
                baz

            [FOO]
            foo
      ",
    )
    .success();
}

#[test]
fn list_displays_recipes_in_submodules() {
  Test::new()
    .write(
      "foo.just",
      "
        bar:
         @echo FOO
      ",
    )
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--list", "--list-submodules"])
    .stdout(
      "
        Available recipes:
            foo:
                bar
      ",
    )
    .success();
}

#[test]
fn modules_are_space_separated_in_output() {
  Test::new()
    .write("foo.just", "foo:")
    .write("bar.just", "bar:")
    .justfile(
      "
        mod foo

        mod bar
      ",
    )
    .args(["--list", "--list-submodules"])
    .stdout(
      "
        Available recipes:
            bar:
                bar

            foo:
                foo
      ",
    )
    .success();
}

#[test]
fn module_recipe_list_alignment_ignores_private_recipes() {
  Test::new()
    .write(
      "foo.just",
      "
        # foos
        foo:
         @echo FOO

        [private]
        barbarbar:
         @echo BAR

        @_bazbazbaz:
         @echo BAZ
      ",
    )
    .justfile("mod foo")
    .args(["--list", "--list-submodules"])
    .stdout(
      "
        Available recipes:
            foo:
                foo # foos
      ",
    )
    .success();
}

#[test]
fn nested_modules_are_properly_indented() {
  Test::new()
    .write("foo.just", "mod bar")
    .write(
      "bar.just",
      "
        baz:
         @echo FOO
      ",
    )
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--list", "--list-submodules"])
    .stdout(
      "
        Available recipes:
            foo:
                bar:
                    baz
      ",
    )
    .success();
}

#[test]
fn module_doc_rendered() {
  Test::new()
    .write("foo.just", "")
    .justfile(
      "
        # Module foo
        mod foo
      ",
    )
    .args(["--list"])
    .stdout(
      "
        Available recipes:
            foo ... # Module foo
      ",
    )
    .success();
}

#[test]
fn module_doc_aligned() {
  Test::new()
    .write("foo.just", "")
    .write("bar.just", "")
    .justfile(
      "
        # Module foo
        mod foo

        # comment
        mod very_long_name_for_module \"bar.just\" # comment

        # will change your world
        recipe:
            @echo Hi
      ",
    )
    .args(["--list"])
    .stdout(
      "
        Available recipes:
            recipe                        # will change your world
            foo ...                       # Module foo
            very_long_name_for_module ... # comment
      ",
    )
    .success();
}

#[test]
fn submodules_without_groups() {
  Test::new()
    .write("foo.just", "")
    .justfile(
      "
        mod foo

        [group: 'baz']
        bar:
      ",
    )
    .args(["--list"])
    .stdout(
      "
        Available recipes:
            foo ...

            [baz]
            bar
      ",
    )
    .success();
}

#[test]
fn no_space_before_submodules_not_following_groups() {
  Test::new()
    .write("foo.just", "")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--list"])
    .stdout(
      "
        Available recipes:
            foo ...
      ",
    )
    .success();
}

#[test]
fn backticks_highlighted() {
  Test::new()
    .justfile(
      "
        # Comment `` `with backticks` and trailing text
        recipe:
      ",
    )
    .args(["--list", "--color=always"])
    .stdout(
      "
                Available recipes:
                    recipe \u{1b}[34m#\u{1b}[0m \u{1b}[34mComment \u{1b}[0m\u{1b}[36m``\u{1b}[0m\u{1b}[34m \u{1b}[0m\u{1b}[36m`with backticks`\u{1b}[0m\u{1b}[34m and trailing text\u{1b}[0m
              ")
            .success();
}

#[test]
fn unclosed_backticks() {
  Test::new()
    .justfile(
      "
        # Comment `with unclosed backtick
        recipe:
      ",
    )
    .args(["--list", "--color=always"])
    .stdout(
      "
        Available recipes:
            recipe \u{1b}[34m#\u{1b}[0m \u{1b}[34mComment \u{1b}[0m\u{1b}[36m`with unclosed backtick\u{1b}[0m
      ")
    .success();
}

#[test]
fn list_submodules_requires_list() {
  Test::new()
    .arg("--list-submodules")
    .stderr_regex(unindent(
      "
        error: the following required arguments were not provided:
          --list .*",
    ))
    .status(2);
}

#[test]
fn options_are_collapsed_in_signature() {
  Test::new()
    .justfile(
      "
        [arg('foo', long)]
        bar foo='baz':
          echo {{foo}}
      ",
    )
    .arg("--list")
    .stdout(
      "
        Available recipes:
            bar [OPTIONS]
      ",
    )
    .success();
}

#[test]
fn positional_and_option_parameters_in_signature() {
  Test::new()
    .justfile(
      "
        [arg('foo', long)]
        [arg('bar', short='b')]
        recipe qux foo='x' bar='y' baz='z':
          echo {{foo}} {{bar}} {{baz}} {{qux}}
      ",
    )
    .arg("--list")
    .stdout(
      "
        Available recipes:
            recipe [OPTIONS] qux baz='z'
      ",
    )
    .success();
}

#[test]
fn doc_above_wide_signature() {
  Test::new()
    .justfile(
      r#"
        # comment
        foo bar="..................................................":
      "#,
    )
    .arg("--list")
    .stdout(
      r#"
        Available recipes:
            # comment
            foo bar=".................................................."
      "#,
    )
    .success();
}
