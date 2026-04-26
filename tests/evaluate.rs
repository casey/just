use super::*;

#[test]
fn evaluate() {
  Test::new()
    .arg("--evaluate")
    .justfile(
      r#"
        foo := "a\t"
        hello := "c"
        bar := "b\t"
        ab := foo + bar + hello

        wut:
          touch /this/is/not/a/file
      "#,
    )
    .stdout(
      r#"ab    := "a	b	c"
bar   := "b	"
foo   := "a	"
hello := "c"
"#,
    )
    .success();
}

#[test]
fn evaluate_empty() {
  Test::new()
    .arg("--evaluate")
    .justfile(
      "
        a := 'foo'
      ",
    )
    .stdout(
      r#"
        a := "foo"
      "#,
    )
    .success();
}

#[test]
fn evaluate_multiple() {
  Test::new()
    .arg("--evaluate")
    .arg("a")
    .arg("c")
    .justfile(
      "
        a := 'x'
        b := 'y'
        c := 'z'
      ",
    )
    .stderr("error: `--evaluate` used with unexpected argument: `c`\n")
    .failure();
}

#[test]
fn evaluate_single_free() {
  Test::new()
    .arg("--evaluate")
    .arg("b")
    .justfile(
      "
        a := 'x'
        b := 'y'
        c := 'z'
      ",
    )
    .stdout("y")
    .success();
}

#[test]
fn evaluate_with_suggestion() {
  Test::new()
    .arg("--evaluate")
    .arg("aby")
    .justfile(
      "
        abc := 'x'
      ",
    )
    .stderr(
      "
        error: justfile does not contain variable or submodule `aby`
        Did you mean `abc`?
      ",
    )
    .failure();
}

#[test]
fn evaluate_no_suggestion() {
  Test::new()
    .arg("--evaluate")
    .arg("goodbye")
    .justfile(
      "
        hello := 'x'
      ",
    )
    .stderr(
      "
        error: justfile does not contain variable or submodule `goodbye`
      ",
    )
    .failure();
}

#[test]
fn evaluate_private() {
  Test::new()
    .arg("--evaluate")
    .justfile(
      "
        [private]
        foo := 'one'
        bar := 'two'
        _baz := 'three'
      ",
    )
    .stdout("bar := \"two\"\n")
    .success();
}

#[test]
fn evaluate_single_private() {
  Test::new()
    .arg("--evaluate")
    .arg("foo")
    .justfile(
      "
        [private]
        foo := 'one'
        bar := 'two'
        _baz := 'three'
      ",
    )
    .stdout("one")
    .success();
}

#[test]
fn evaluate_variable_chosen_over_submodule() {
  Test::new()
    .write("foo.just", "bar:\n")
    .justfile(
      "
        mod foo

        foo := 'bar'
      ",
    )
    .args(["--evaluate", "foo"])
    .stdout("bar")
    .success();
}

#[test]
fn evaluate_submodule_chosen_over_variable_in_path() {
  Test::new()
    .write("foo.just", "a := 'x'\n")
    .justfile(
      "
        mod foo

        foo := 'bar'
      ",
    )
    .args(["--evaluate", "foo::a"])
    .stdout("x")
    .success();
}

#[test]
fn evaluate_submodule() {
  Test::new()
    .write("foo.just", "a := 'x'\nb := 'y'\n")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--evaluate", "foo"])
    .stdout("a := \"x\"\nb := \"y\"\n")
    .success();
}

#[test]
fn evaluate_variable_in_submodule() {
  Test::new()
    .write("foo.just", "a := 'x'\nb := 'y'\n")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--evaluate", "foo::a"])
    .stdout("x")
    .success();
}

#[test]
fn evaluate_unknown_submodule_with_suggestion() {
  Test::new()
    .write("foo.just", "a := 'x'\n")
    .justfile(
      "
        mod foo
      ",
    )
    .args(["--evaluate", "fob::a"])
    .stderr(
      "
        error: justfile does not contain submodule `fob`
        Did you mean `foo`?
      ",
    )
    .failure();
}

#[test]
fn dont_evaluate_unnecessary_variables() {
  Test::new()
    .justfile(
      "
        x := 'FOO'

        y := `exit 1`
      ",
    )
    .args(["--evaluate", "x"])
    .stdout("FOO")
    .success();
}

#[test]
fn format_shell() {
  Test::new()
    .args(["--evaluate", "--evaluate-format", "shell"])
    .justfile(
      "
        foo := 'a'
        hello := 'b'
        bar := 'c'
      ",
    )
    .stdout(
      r#"
        bar="c"
        foo="a"
        hello="b"
      "#,
    )
    .success();
}

#[test]
fn format_shell_special_characters_are_escaped() {
  Test::new()
    .args(["--evaluate", "--evaluate-format", "shell"])
    .justfile(
      r#"
        foo := '!"$\`'
      "#,
    )
    .stdout(
      r#"
        foo="\!\"\$\\\`"
      "#,
    )
    .success();
}

#[test]
fn format_shell_exported_variables_are_exported() {
  Test::new()
    .args(["--evaluate", "--evaluate-format", "shell"])
    .justfile(
      "
        export foo := 'bar'
      ",
    )
    .stdout(
      r#"
        export foo="bar"
      "#,
    )
    .success();
}

#[test]
fn format_shell_variables_are_exported_with_setting() {
  Test::new()
    .args(["--evaluate", "--evaluate-format", "shell"])
    .justfile(
      "
        set export

        foo := 'bar'
      ",
    )
    .stdout(
      r#"
        export foo="bar"
      "#,
    )
    .success();
}

#[test]
fn format_shell_dashes_in_variables_are_replaced_with_underscores() {
  Test::new()
    .args(["--evaluate", "--evaluate-format", "shell"])
    .justfile(
      "
        foo-bar := 'baz'
      ",
    )
    .stdout(
      r#"
        foo_bar="baz"
      "#,
    )
    .success();
}
