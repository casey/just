use super::*;

#[test]
fn show() {
  Test::new()
    .arg("--show")
    .arg("recipe")
    .justfile(
      r#"hello := "foo"
bar := hello + hello
recipe:
 echo {{hello + "bar" + bar}}"#,
    )
    .stdout(
      r#"
    recipe:
        echo {{ hello + "bar" + bar }}
  "#,
    )
    .run();
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
    .run();
}

#[test]
fn alias_show_missing_target() {
  Test::new()
    .arg("--show")
    .arg("f")
    .justfile("alias f := foo")
    .status(EXIT_FAILURE)
    .stderr(
      "
    error: Alias `f` has an unknown target `foo`
     ——▶ justfile:1:7
      │
    1 │ alias f := foo
      │       ^
  ",
    )
    .run();
}

#[test]
fn show_suggestion() {
  Test::new()
    .arg("--show")
    .arg("hell")
    .justfile(
      r#"
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

a Z="\t z":
"#,
    )
    .stderr("error: Justfile does not contain recipe `hell`\nDid you mean `hello`?\n")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn show_alias_suggestion() {
  Test::new()
    .arg("--show")
    .arg("fo")
    .justfile(
      r#"
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

alias foo := hello

a Z="\t z":
"#,
    )
    .stderr(
      "
    error: Justfile does not contain recipe `fo`
    Did you mean `foo`, an alias for `hello`?
  ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn show_no_suggestion() {
  Test::new()
    .arg("--show")
    .arg("hell")
    .justfile(
      r#"
helloooooo a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

a Z="\t z":
"#,
    )
    .stderr("error: Justfile does not contain recipe `hell`\n")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn show_no_alias_suggestion() {
  Test::new()
    .arg("--show")
    .arg("fooooooo")
    .justfile(
      r#"
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

alias foo := hello

a Z="\t z":
"#,
    )
    .stderr("error: Justfile does not contain recipe `fooooooo`\n")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn show_recipe_at_path() {
  Test::new()
    .write("foo.just", "bar:\n @echo MODULE")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--show", "foo::bar"])
    .stdout("bar:\n    @echo MODULE\n")
    .run();
}

#[test]
fn show_invalid_path() {
  Test::new()
    .args(["--show", "$hello"])
    .stderr("error: Invalid module path `$hello`\n")
    .status(1)
    .run();
}

#[test]
fn show_space_separated_path() {
  Test::new()
    .write("foo.just", "bar:\n @echo MODULE")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--show", "foo bar"])
    .stdout("bar:\n    @echo MODULE\n")
    .run();
}
