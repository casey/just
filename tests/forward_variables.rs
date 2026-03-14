use super::*;

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
fn no_overrides_returns_empty() {
  Test::new()
    .justfile(
      r#"
      v1 := 'x'
      fwd := forward_variables()
    "#,
    )
    .args(["--evaluate", "fwd"])
    .stdout("")
    .success();
}

#[test]
fn single_override() {
  Test::new()
    .justfile(
      r#"
      v1 := 'x'
      fwd := forward_variables()
    "#,
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
      r#"
      v1 := 'a'
      v2 := 'b'
      fwd := forward_variables()
    "#,
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
      r#"
      greeting := 'hi'
      fwd := forward_variables()
    "#,
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
      r#"
      msg := 'hi'
      fwd := forward_variables()
    "#,
    )
    .args(["--set", "msg", "it's"])
    .args(["--evaluate", "fwd"])
    .stdout(r#"msg='it'\''s'"#)
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
fn forward_single_quote_to_child_process() {
  Test::new()
    .justfile(
      r#"
      v1 := "x"

      a:
          @echo {{ quote(v1) }}

      c:
          @{{ just_executable() }} --justfile {{ justfile() }} {{ forward_variables() }} a
    "#,
    )
    .args(["--set", "v1", "'"])
    .arg("c")
    .stdout("'\n")
    .stderr_regex(".*")
    .success();
}

#[test]
fn empty_value() {
  Test::new()
    .justfile(
      r#"
      v1 := 'x'
      fwd := forward_variables()
    "#,
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
      r#"
      v1 := 'a'
      v2 := 'b'
      fwd := forward_variables('v1')
    "#,
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
      r#"
      v1 := 'a'
      v2 := 'b'
      v3 := 'c'
      fwd := forward_variables('v1', 'v3')
    "#,
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
      r#"
      v1 := 'a'
      v2 := 'b'
      fwd := forward_variables('v1', 'v2')
    "#,
    )
    .args(["--set", "v1", "hello"])
    .args(["--evaluate", "fwd"])
    .stdout("v1='hello'")
    .success();
}

#[test]
fn value_with_equals_sign() {
  Test::new()
    .justfile(
      r#"
      v1 := 'x'
      fwd := forward_variables()
    "#,
    )
    .args(["--set", "v1", "a=b=c"])
    .args(["--evaluate", "fwd"])
    .stdout("v1='a=b=c'")
    .success();
}

#[test]
fn value_with_special_shell_chars() {
  Test::new()
    .justfile(
      r#"
      v1 := 'x'
      fwd := forward_variables()
    "#,
    )
    .args(["--set", "v1", r#"$HOME `echo hi` "quoted""#])
    .args(["--evaluate", "fwd"])
    .stdout(r#"v1='$HOME `echo hi` "quoted"'"#)
    .success();
}

#[test]
fn value_with_backslash() {
  Test::new()
    .justfile(
      r#"
      v1 := 'x'
      fwd := forward_variables()
    "#,
    )
    .args(["--set", "v1", r#"path\to\file"#])
    .args(["--evaluate", "fwd"])
    .stdout(r#"v1='path\to\file'"#)
    .success();
}

#[test]
fn value_with_newline() {
  Test::new()
    .justfile(
      r#"
      v1 := 'x'
      fwd := forward_variables()
    "#,
    )
    .args(["--set", "v1", "line1\nline2"])
    .args(["--evaluate", "fwd"])
    .stdout("v1='line1\nline2'")
    .success();
}

#[test]
fn selective_all_miss() {
  Test::new()
    .justfile(
      r#"
      v1 := 'a'
      v2 := 'b'
      fwd := forward_variables('v1', 'v2')
    "#,
    )
    .args(["--evaluate", "fwd"])
    .stdout("")
    .success();
}

#[test]
fn value_with_multiple_single_quotes() {
  Test::new()
    .justfile(
      r#"
      v1 := 'x'
      fwd := forward_variables()
    "#,
    )
    .args(["--set", "v1", "it's a 'test'"])
    .args(["--evaluate", "fwd"])
    .stdout(r#"v1='it'\''s a '\''test'\'''"#)
    .success();
}

#[test]
fn value_with_mixed_quotes() {
  Test::new()
    .justfile(
      r#"
      v1 := 'x'
      fwd := forward_variables()
    "#,
    )
    .args(["--set", "v1", r#"he said "it's fine""#])
    .args(["--evaluate", "fwd"])
    .stdout(r#"v1='he said "it'\''s fine"'"#)
    .success();
}

#[test]
fn value_with_consecutive_single_quotes() {
  Test::new()
    .justfile(
      r#"
      v1 := 'x'
      fwd := forward_variables()
    "#,
    )
    .args(["--set", "v1", "''"])
    .args(["--evaluate", "fwd"])
    .stdout(r#"v1=''\'''\'''"#)
    .success();
}

// ── forward_variables_with(sep, prefix, kvsep, names...) ──

#[test]
fn with_psql_format() {
  Test::new()
    .justfile(
      r#"
      table_name := 'users'
      uid := '42'
      fwd := forward_variables_with(' ', '-v ', '=', 'table_name', 'uid')
    "#,
    )
    .args(["--set", "table_name", "orders"])
    .args(["--set", "uid", "100"])
    .args(["--evaluate", "fwd"])
    .stdout("-v table_name='orders' -v uid='100'")
    .success();
}

#[test]
fn with_docker_build_arg_format() {
  Test::new()
    .justfile(
      r#"
      node_env := 'dev'
      fwd := forward_variables_with(' ', '--build-arg ', '=', 'node_env')
    "#,
    )
    .args(["--set", "node_env", "production"])
    .args(["--evaluate", "fwd"])
    .stdout("--build-arg node_env='production'")
    .success();
}

#[test]
fn with_cmake_format() {
  Test::new()
    .justfile(
      r#"
      build_type := 'Debug'
      fwd := forward_variables_with(' ', '-D', '=', 'build_type')
    "#,
    )
    .args(["--set", "build_type", "Release"])
    .args(["--evaluate", "fwd"])
    .stdout("-Dbuild_type='Release'")
    .success();
}

#[test]
fn with_all_overrides_no_names() {
  Test::new()
    .justfile(
      r#"
      v1 := 'a'
      v2 := 'b'
      fwd := forward_variables_with(' ', '-v ', '=')
    "#,
    )
    .args(["--set", "v1", "x"])
    .args(["--set", "v2", "y"])
    .args(["--evaluate", "fwd"])
    .stdout("-v v1='x' -v v2='y'")
    .success();
}

#[test]
fn with_no_overrides_returns_empty() {
  Test::new()
    .justfile(
      r#"
      v1 := 'a'
      fwd := forward_variables_with(' ', '-v ', '=')
    "#,
    )
    .args(["--evaluate", "fwd"])
    .stdout("")
    .success();
}

#[test]
fn with_empty_prefix() {
  Test::new()
    .justfile(
      r#"
      v1 := 'a'
      fwd := forward_variables_with(' ', '', '=', 'v1')
    "#,
    )
    .args(["--set", "v1", "hello"])
    .args(["--evaluate", "fwd"])
    .stdout("v1='hello'")
    .success();
}

#[test]
fn with_value_containing_single_quotes() {
  Test::new()
    .justfile(
      r#"
      msg := 'hi'
      fwd := forward_variables_with(' ', '-v ', '=', 'msg')
    "#,
    )
    .args(["--set", "msg", "it's"])
    .args(["--evaluate", "fwd"])
    .stdout(r#"-v msg='it'\''s'"#)
    .success();
}

#[test]
fn with_custom_kvsep() {
  Test::new()
    .justfile(
      r#"
      k1 := 'a'
      fwd := forward_variables_with(' ', '', ':', 'k1')
    "#,
    )
    .args(["--set", "k1", "val"])
    .args(["--evaluate", "fwd"])
    .stdout("k1:'val'")
    .success();
}

#[test]
fn with_selective_missing_override() {
  Test::new()
    .justfile(
      r#"
      v1 := 'a'
      v2 := 'b'
      fwd := forward_variables_with(' ', '-v ', '=', 'v1', 'v2')
    "#,
    )
    .args(["--set", "v1", "hello"])
    .args(["--evaluate", "fwd"])
    .stdout("-v v1='hello'")
    .success();
}

#[test]
fn with_forward_to_child_process() {
  Test::new()
    .justfile(
      r#"
      v1 := "x"

      a:
          @echo {{ v1 }}

      c:
          @{{ just_executable() }} --justfile {{ justfile() }} {{ forward_variables_with(' ', '', '=', 'v1') }} a
    "#,
    )
    .arg("v1=y")
    .arg("c")
    .stdout("y\n")
    .stderr_regex(".*")
    .success();
}
