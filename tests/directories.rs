use super::*;

#[test]
fn cache_directory() {
  assert_eval_eq(
    "cache_directory()",
    dirs::cache_dir().unwrap_or_default().to_str().unwrap(),
  );
}

#[test]
fn config_directory() {
  assert_eval_eq(
    "config_directory()",
    dirs::config_dir().unwrap_or_default().to_str().unwrap(),
  );
}

#[test]
fn config_local_directory() {
  assert_eval_eq(
    "config_local_directory()",
    dirs::config_local_dir()
      .unwrap_or_default()
      .to_str()
      .unwrap(),
  );
}

#[test]
fn data_directory() {
  assert_eval_eq(
    "data_directory()",
    dirs::data_dir().unwrap_or_default().to_str().unwrap(),
  );
}

#[test]
fn data_local_directory() {
  assert_eval_eq(
    "data_local_directory()",
    dirs::data_local_dir().unwrap_or_default().to_str().unwrap(),
  );
}

#[test]
fn executable_directory() {
  if let Some(executable_dir) = dirs::executable_dir() {
    assert_eval_eq("executable_directory()", executable_dir.to_str().unwrap());
  } else {
    Test::new()
      .justfile("x := executable_directory()")
      .args(["--evaluate", "x"])
      .stderr(
        "
          error: call to function `executable_directory` failed: executable directory not found
           ——▶ justfile:1:6
            │
          1 │ x := executable_directory()
            │      ^^^^^^^^^^^^^^^^^^^^
        ",
      )
      .failure();
  }
}

#[test]
fn home_directory() {
  assert_eval_eq(
    "home_directory()",
    dirs::home_dir().unwrap_or_default().to_str().unwrap(),
  );
}

#[test]
fn runtime_directory() {
  if cfg!(not(target_os = "linux")) {
    return;
  }

  assert_eval_eq(
    "runtime_directory()",
    dirs::runtime_dir().unwrap_or_default().to_str().unwrap(),
  );
}

#[test]
fn runtime_directory_not_found() {
  if cfg!(target_os = "linux") {
    return;
  }

  Test::new()
    .justfile("x := runtime_directory()")
    .args(["--evaluate", "x"])
    .stderr(
      "
        error: call to function `runtime_directory` failed: runtime directory not found
         ——▶ justfile:1:6
          │
        1 │ x := runtime_directory()
          │      ^^^^^^^^^^^^^^^^^
      ",
    )
    .failure();
}
