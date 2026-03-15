use super::*;

#[test]
fn unknown_override() {
  Test::new()
    .justfile(
      "
        a:
          echo {{`f() { return 100; }; f`}}
      ",
    )
    .args(["foo=bar", "baz=bob", "a"])
    .stderr(
      "
        error: Variables `baz` and `foo` overridden on the command line but not present in justfile
      ",
    )
    .failure();
}

#[test]
fn unknown_override_options() {
  Test::new()
    .arg("--set")
    .arg("foo")
    .arg("bar")
    .arg("--set")
    .arg("baz")
    .arg("bob")
    .arg("--set")
    .arg("a")
    .arg("b")
    .arg("a")
    .arg("b")
    .justfile(
      "foo:
 echo hello
 echo {{`exit 111`}}
a := `exit 222`",
    )
    .stderr(
      "error: Variables `baz` and `foo` overridden on the command line but not present \
    in justfile\n",
    )
    .failure();
}

#[test]
fn unknown_override_args() {
  Test::new()
    .arg("foo=bar")
    .arg("baz=bob")
    .arg("a=b")
    .arg("a")
    .arg("b")
    .justfile(
      "foo:
 echo hello
 echo {{`exit 111`}}
a := `exit 222`",
    )
    .stderr(
      "error: Variables `baz` and `foo` overridden on the command line but not present \
    in justfile\n",
    )
    .failure();
}

#[test]
fn unknown_override_arg() {
  Test::new()
    .arg("foo=bar")
    .arg("a=b")
    .arg("a")
    .arg("b")
    .justfile(
      "foo:
 echo hello
 echo {{`exit 111`}}
a := `exit 222`",
    )
    .stderr("error: Variable `foo` overridden on the command line but not present in justfile\n")
    .failure();
}

#[test]
fn overrides_first() {
  Test::new()
    .arg("foo=bar")
    .arg("a=b")
    .arg("recipe")
    .arg("baz=bar")
    .justfile(
      r#"
foo := "foo"
a := "a"
baz := "baz"

recipe arg:
 echo arg={{arg}}
 echo {{foo + a + baz}}"#,
    )
    .stdout("arg=baz=bar\nbarbbaz\n")
    .stderr("echo arg=baz=bar\necho barbbaz\n")
    .success();
}

#[test]
fn overrides_not_evaluated() {
  Test::new()
    .arg("foo=bar")
    .arg("a=b")
    .arg("recipe")
    .arg("baz=bar")
    .justfile(
      r#"
foo := `exit 1`
a := "a"
baz := "baz"

recipe arg:
 echo arg={{arg}}
 echo {{foo + a + baz}}"#,
    )
    .stdout("arg=baz=bar\nbarbbaz\n")
    .stderr("echo arg=baz=bar\necho barbbaz\n")
    .success();
}
