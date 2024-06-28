use super::*;

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
    .run();
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
    .run();

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
    .run();

  Test::new()
    .write("a.just", "a:\nimport 'e.just'")
    .write("b.just", "b:\nimport 'f.just'")
    .write("c.just", "c:\nimport 'g.just'")
    .write("d.just", "d:\nimport 'h.just'")
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
    .run();

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
    .run();
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
    .run();
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
    .run();

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
    .run();
}

#[test]
fn list_invalid_path() {
  Test::new()
    .args(["--list", "$hello"])
    .stderr("error: Invalid module path `$hello`\n")
    .status(1)
    .run();
}

#[test]
fn list_unknown_submodule() {
  Test::new()
    .args(["--list", "hello"])
    .stderr("error: Justfile does not contain submodule nor group `hello`\n")
    .status(1)
    .run();
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
    .write("bar.just", "[group('BAZ')]\nbaz:")
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
    .run();
}

#[test]
fn list_displays_recipes_in_submodules() {
  Test::new()
    .write("foo.just", "bar:\n @echo FOO")
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
    .run();
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
    .run();
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
    .run();
}

#[test]
fn nested_modules_are_properly_indented() {
  Test::new()
    .write("foo.just", "mod bar")
    .write("bar.just", "baz:\n @echo FOO")
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
    .run();
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
    .run();
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

        # another lifechanging experience
        recipe2:
            @echo fooled

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
            recipe2                       # another lifechanging experience
            foo ...                       # Module foo
            very_long_name_for_module ... # comment
      ",
    )
    .run();
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
    .run();
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
    .run();
}
#[test]
fn group_list() {
  Test::new()
    .justfile(
      "
        [group('group_name')]
        recipe:
            @echo Hi

        recipe2:
            @echo Hi
      ",
    )
    .test_round_trip(false)
    .args(["--list", "group_name"])
    .stdout(
      "
        Available recipes:
            recipe
      ",
    )
    .run();
}

#[test]
fn group_list_recursive() {
  Test::new()
    .write(
      "foo.just",
      "
[group('group_name')]
rec_recipe:
    @echo recursion fun
      ",
    )
    .justfile(
      "
        mod foo

        [group('group_name')]
        recipe:
            @echo Hi

        recipe2:
            @echo Hi
      ",
    )
    .test_round_trip(false)
    .args(["--list-submodules", "--list", "group_name"])
    .stdout(
      "
        Available recipes:
            recipe
            foo::rec_recipe
      ",
    )
    .run();
}

#[test]
fn group_list_recursive_with_comments() {
  Test::new()
    .write(
      "foo.just",
      "
# this is a module comment
[group('group_name')]
rec_recipe:
    @echo recursion fun
      ",
    )
    .justfile(
      "
        mod foo

        # comment
        [group('group_name')]
        recipe:
            @echo Hi

        recipe2:
            @echo Hi
      ",
    )
    .test_round_trip(false)
    .args(["--list-submodules", "--list", "group_name"])
    .stdout(
      "
        Available recipes:
            recipe          # comment
            foo::rec_recipe # this is a module comment
      ",
    )
    .run();
}
#[test]
fn group_list_recursive_nested() {
  Test::new()
    .write(
      "bar.just",
      "
# this is a module comment
[group('group_name')]
rec_recipe:
    @echo recursion fun

mod baz
      ",
    )
    .write(
      "foo.just",
      "
mod bar
      ",
    )
    .write(
      "baz.just",
      "
[group('group_name')]
other:
    @echo recursion fun
     ",
    )
    .justfile(
      "
        mod foo

        # comment
        [group('group_name')]
        recipe:
            @echo Hi
      ",
    )
    .test_round_trip(false)
    .args(["--list-submodules", "--list", "group_name"])
    .stdout(
      "
        Available recipes:
            recipe               # comment
            foo::bar::rec_recipe # this is a module comment
            foo::bar::baz::other
      ",
    )
    .run();
}
