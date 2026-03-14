use super::*;

#[test]
fn no_overrides_returns_empty() {
  Test::new()
    .justfile(
      "
      v1 := 'x'
      fwd := forward_variables()
    ",
    )
    .args(["--evaluate", "fwd"])
    .stdout("")
    .success();
}

#[test]
fn single_override() {
  Test::new()
    .justfile(
      "
      v1 := 'x'
      fwd := forward_variables()
    ",
    )
    .args(["--set", "v1", "y"])
    .args(["--evaluate", "fwd"])
    .stdout("v1='y'")
    .success();
}

#[test]
fn multiple_overrides() {
  Test::new()
    .justfile(
      "
      v1 := 'a'
      v2 := 'b'
      fwd := forward_variables()
    ",
    )
    .args(["--set", "v1", "hello"])
    .args(["--set", "v2", "world"])
    .args(["--evaluate", "fwd"])
    .stdout("v1='hello' v2='world'")
    .success();
}

#[test]
fn value_with_spaces() {
  Test::new()
    .justfile(
      "
      greeting := 'hi'
      fwd := forward_variables()
    ",
    )
    .args(["--set", "greeting", "hello world"])
    .args(["--evaluate", "fwd"])
    .stdout("greeting='hello world'")
    .success();
}

#[test]
fn value_with_single_quotes() {
  Test::new()
    .justfile(
      "
      msg := 'hi'
      fwd := forward_variables()
    ",
    )
    .args(["--set", "msg", "it's"])
    .args(["--evaluate", "fwd"])
    .stdout("msg='it'\\''s'")
    .success();
}

#[test]
fn forward_to_child_process() {
  Test::new()
    .justfile(
      r#"
      v1 := "x"

      a:
          @echo {{ v1 }}

      c:
          @{{ just_executable() }} --justfile {{ justfile() }} {{ forward_variables() }} a
    "#,
    )
    .arg("v1=y")
    .arg("c")
    .stdout("y\n")
    .stderr_regex(".*")
    .success();
}

#[test]
fn no_forward_without_function() {
  Test::new()
    .justfile(
      r#"
      v1 := "x"

      a:
          @echo {{ v1 }}

      c:
          @{{ just_executable() }} --justfile {{ justfile() }} a
    "#,
    )
    .arg("v1=y")
    .arg("c")
    .stdout("x\n")
    .stderr_regex(".*")
    .success();
}

#[test]
fn child_explicit_override_wins() {
  Test::new()
    .justfile(
      r#"
      v1 := "x"

      a:
          @echo {{ v1 }}

      c:
          @{{ just_executable() }} --justfile {{ justfile() }} {{ forward_variables() }} v1=z a
    "#,
    )
    .arg("v1=y")
    .arg("c")
    .stdout("z\n")
    .stderr_regex(".*")
    .success();
}

#[test]
fn with_set_flag() {
  Test::new()
    .justfile(
      r#"
      v1 := "x"

      a:
          @echo {{ v1 }}

      c:
          @{{ just_executable() }} --justfile {{ justfile() }} {{ forward_variables() }} a
    "#,
    )
    .args(["--set", "v1", "y"])
    .arg("c")
    .stdout("y\n")
    .stderr_regex(".*")
    .success();
}

#[test]
fn empty_value() {
  Test::new()
    .justfile(
      "
      v1 := 'x'
      fwd := forward_variables()
    ",
    )
    .args(["--set", "v1", ""])
    .args(["--evaluate", "fwd"])
    .stdout("v1=''")
    .success();
}

#[test]
fn selective_single() {
  Test::new()
    .justfile(
      "
      v1 := 'a'
      v2 := 'b'
      fwd := forward_variables('v1')
    ",
    )
    .args(["--set", "v1", "hello"])
    .args(["--set", "v2", "world"])
    .args(["--evaluate", "fwd"])
    .stdout("v1='hello'")
    .success();
}

#[test]
fn selective_multiple() {
  Test::new()
    .justfile(
      "
      v1 := 'a'
      v2 := 'b'
      v3 := 'c'
      fwd := forward_variables('v1', 'v3')
    ",
    )
    .args(["--set", "v1", "x"])
    .args(["--set", "v2", "y"])
    .args(["--set", "v3", "z"])
    .args(["--evaluate", "fwd"])
    .stdout("v1='x' v3='z'")
    .success();
}

#[test]
fn selective_missing_override() {
  Test::new()
    .justfile(
      "
      v1 := 'a'
      v2 := 'b'
      fwd := forward_variables('v1', 'v2')
    ",
    )
    .args(["--set", "v1", "hello"])
    .args(["--evaluate", "fwd"])
    .stdout("v1='hello'")
    .success();
}
