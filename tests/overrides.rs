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
      "
foo := 'foo'
a := 'a'
baz := 'baz'

recipe arg:
 echo arg={{arg}}
 echo {{foo + a + baz}}",
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
      "
foo := `exit 1`
a := 'a'
baz := 'baz'

recipe arg:
 echo arg={{arg}}
 echo {{foo + a + baz}}",
    )
    .stdout("arg=baz=bar\nbarbbaz\n")
    .stderr("echo arg=baz=bar\necho barbbaz\n")
    .success();
}

#[test]
fn invalid_override_path_set() {
  Test::new()
    .arg("--set")
    .arg("0::foo")
    .arg("bar")
    .stderr("error: Invalid override path `0::foo`\n")
    .failure();
}

#[test]
fn invalid_override_path_positional() {
  Test::new()
    .arg("0::foo=bar")
    .stderr("error: Invalid override path `0::foo`\n")
    .failure();
}

#[test]
fn unknown_variable_in_submodule_override() {
  Test::new()
    .justfile("mod foo")
    .write("foo.just", "bar:\n @echo bar")
    .arg("foo::x=b")
    .arg("foo::bar")
    .stderr("error: Variable `foo::x` overridden on the command line but not present in justfile\n")
    .failure();
}

#[test]
fn override_variable_in_submodule() {
  Test::new()
    .justfile("mod foo")
    .write("foo.just", "x := 'a'\nbar:\n @echo {{x}}")
    .arg("foo::x=b")
    .arg("foo::bar")
    .stdout("b\n")
    .success();
}

#[test]
fn override_variable_in_nested_submodule() {
  Test::new()
    .justfile("mod foo")
    .write("foo/mod.just", "mod bar")
    .write("foo/bar.just", "x := 'a'\nbaz:\n @echo {{x}}")
    .arg("foo::bar::x=b")
    .arg("foo::bar::baz")
    .stdout("b\n")
    .success();
}

#[test]
fn override_variable_used_in_setting() {
  Test::new()
    .justfile(
      "
        dir := 'foo'
        set working-directory := dir
        bar:
          @cat file.txt
      ",
    )
    .write("baz/file.txt", "BAZ")
    .arg("dir=baz")
    .arg("bar")
    .stdout("BAZ")
    .success();
}

#[test]
fn submodule_override_does_not_affect_parent() {
  Test::new()
    .justfile(
      "
        mod foo
        x := 'root'
        bar:
          @echo {{x}}
      ",
    )
    .write("foo.just", "x := 'a'\nbaz:\n @echo {{x}}")
    .arg("foo::x=b")
    .arg("bar")
    .stdout("root\n")
    .success();
}

#[test]
fn unknown_submodule_in_override_path() {
  Test::new()
    .arg("foo::x=b")
    .stderr("error: Variable `foo::x` overridden on the command line but not present in justfile\n")
    .failure();
}

#[test]
fn submodule_override_not_evaluated() {
  Test::new()
    .justfile("mod foo")
    .write("foo.just", "x := `exit 1`\nbar:\n @echo {{x}}")
    .arg("foo::x=b")
    .arg("foo::bar")
    .stdout("b\n")
    .success();
}
