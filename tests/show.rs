use super::*;

#[test]
fn search_directory_without_recipe() {
  Test::new()
    .justfile("foo:")
    .args(["--show", "."])
    .stderr("error: `--show` requires recipe\n")
    .failure();
}

#[test]
fn show() {
  Test::new()
    .arg("--show")
    .arg("recipe")
    .justfile(
      "hello := 'foo'
bar := hello + hello
recipe:
 echo {{hello + 'bar' + bar}}",
    )
    .stdout(
      "
        recipe:
            echo {{ hello + 'bar' + bar }}
      ",
    )
    .success();
}

#[test]
fn alias_show() {
  Test::new()
    .arg("--show")
    .arg("f")
    .justfile("foo:\n    bar\nalias f := foo")
    .stdout(
      "
        alias f := foo
        foo:
            bar
      ",
    )
    .success();
}

#[test]
fn alias_show_missing_target() {
  Test::new()
    .arg("--show")
    .arg("f")
    .justfile("alias f := foo")
    .stderr(
      "
        error: alias `f` has an unknown target `foo`
         ——▶ justfile:1:7
          │
        1 │ alias f := foo
          │       ^
      ",
    )
    .failure();
}

#[test]
fn show_suggestion() {
  Test::new()
    .arg("--show")
    .arg("hell")
    .justfile(
      "
        hello:

        a:
      ",
    )
    .stderr("error: justfile does not contain recipe `hell`\nDid you mean `hello`?\n")
    .failure();
}

#[test]
fn show_alias_suggestion() {
  Test::new()
    .arg("--show")
    .arg("fo")
    .justfile(
      "
        hello:

        alias foo := hello

        a:
      ",
    )
    .stderr(
      "
        error: justfile does not contain recipe `fo`
        Did you mean `foo`, an alias for `hello`?
      ",
    )
    .failure();
}

#[test]
fn show_no_suggestion() {
  Test::new()
    .arg("--show")
    .arg("hell")
    .justfile(
      "
        helloooooo:

        a:
      ",
    )
    .stderr("error: justfile does not contain recipe `hell`\n")
    .failure();
}

#[test]
fn show_no_alias_suggestion() {
  Test::new()
    .arg("--show")
    .arg("fooooooo")
    .justfile(
      "
        hello:

        alias foo := hello

        a:
      ",
    )
    .stderr("error: justfile does not contain recipe `fooooooo`\n")
    .failure();
}

#[test]
fn show_recipe_at_path() {
  Test::new()
    .write(
      "foo.just",
      "
        bar:
         @echo MODULE
      ",
    )
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--show", "foo::bar"])
    .stdout("bar:\n    @echo MODULE\n")
    .success();
}

#[test]
fn show_invalid_path() {
  Test::new()
    .args(["--show", "$hello"])
    .stderr("error: invalid module path `$hello`\n")
    .failure();
}

#[test]
fn show_space_separated_path() {
  Test::new()
    .write(
      "foo.just",
      "
        bar:
         @echo MODULE
      ",
    )
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--show", "foo bar"])
    .stdout("bar:\n    @echo MODULE\n")
    .success();
}

#[test]
fn show_recipe_in_search_directory() {
  Test::new()
    .justfile("foo:\n @echo ROOT")
    .write(
      "child/justfile",
      "
        foo:
         @echo CHILD
      ",
    )
    .current_dir("child")
    .args(["--show", "../foo"])
    .stdout("foo:\n    @echo ROOT\n")
    .success();
}

#[test]
fn show_cross_module_dependencies() {
  Test::new()
    .justfile(
      "
        mod sub

        foo: sub::deep
            @echo foo
      ",
    )
    .write(
      "sub.just",
      "
        deep:
            @echo deep
      ",
    )
    .args(["--show", "foo"])
    .stdout(
      "
        foo: sub::deep
            @echo foo
      ",
    )
    .success();
}
