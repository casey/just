use super::*;
use etcetera::BaseStrategy;

#[test]
fn cache_directory() {
  Test::new()
    .justfile("x := cache_directory()")
    .args(["--evaluate", "x"])
    .stdout(dirs::cache_dir().unwrap_or_default().to_string_lossy())
    .run();
}

#[test]
fn config_directory() {
  Test::new()
    .justfile("x := config_directory()")
    .args(["--evaluate", "x"])
    .stdout(dirs::config_dir().unwrap_or_default().to_string_lossy())
    .run();
}

#[test]
fn config_local_directory() {
  Test::new()
    .justfile("x := config_local_directory()")
    .args(["--evaluate", "x"])
    .stdout(
      dirs::config_local_dir()
        .unwrap_or_default()
        .to_string_lossy(),
    )
    .run();
}

#[test]
fn data_directory() {
  Test::new()
    .justfile("x := data_directory()")
    .args(["--evaluate", "x"])
    .stdout(dirs::data_dir().unwrap_or_default().to_string_lossy())
    .run();
}

#[test]
fn data_local_directory() {
  Test::new()
    .justfile("x := data_local_directory()")
    .args(["--evaluate", "x"])
    .stdout(dirs::data_local_dir().unwrap_or_default().to_string_lossy())
    .run();
}

#[test]
fn executable_directory() {
  if let Some(executable_dir) = dirs::executable_dir() {
    Test::new()
      .justfile("x := executable_directory()")
      .args(["--evaluate", "x"])
      .stdout(executable_dir.to_string_lossy())
      .run();
  } else {
    Test::new()
      .justfile("x := executable_directory()")
      .args(["--evaluate", "x"])
      .stderr(
        "
          error: Call to function `executable_directory` failed: executable directory not found
           ——▶ justfile:1:6
            │
          1 │ x := executable_directory()
            │      ^^^^^^^^^^^^^^^^^^^^
        ",
      )
      .status(EXIT_FAILURE)
      .run();
  }
}

#[test]
fn home_directory() {
  Test::new()
    .justfile("x := home_directory()")
    .args(["--evaluate", "x"])
    .stdout(dirs::home_dir().unwrap_or_default().to_string_lossy())
    .run();
}

#[test]
fn xdg_cache_directory() {
  Test::new()
    .justfile("x := xdg_cache_directory()")
    .args(["--evaluate", "x"])
    .stdout(
      etcetera::choose_base_strategy()
        .unwrap()
        .cache_dir()
        .to_string_lossy(),
    )
    .run();
}

#[test]
fn xdg_config_directory() {
  Test::new()
    .justfile("x := xdg_config_directory()")
    .args(["--evaluate", "x"])
    .stdout(
      etcetera::choose_base_strategy()
        .unwrap()
        .config_dir()
        .to_string_lossy(),
    )
    .run();
}

#[test]
fn xdg_data_directory() {
  Test::new()
    .justfile("x := xdg_data_directory()")
    .args(["--evaluate", "x"])
    .stdout(
      etcetera::choose_base_strategy()
        .unwrap()
        .data_dir()
        .to_string_lossy(),
    )
    .run();
}

#[test]
fn xdg_home_directory() {
  Test::new()
    .justfile("x := xdg_home_directory()")
    .args(["--evaluate", "x"])
    .stdout(
      etcetera::choose_base_strategy()
        .unwrap()
        .home_dir()
        .to_string_lossy(),
    )
    .run();
}

#[test]
fn xdg_runtime_directory() {
  if let Some(runtime_dir) = etcetera::choose_base_strategy().unwrap().runtime_dir() {
    Test::new()
      .justfile("x := xdg_runtime_directory()")
      .args(["--evaluate", "x"])
      .stdout(runtime_dir.to_string_lossy())
      .run();
  } else {
    Test::new()
      .justfile("x := xdg_runtime_directory()")
      .args(["--evaluate", "x"])
      .stderr(
        "
          error: Call to function `xdg_runtime_directory` failed: runtime directory not found
           ——▶ justfile:1:6
            │
          1 │ x := xdg_runtime_directory()
            │      ^^^^^^^^^^^^^^^^^^^^^
        ",
      )
      .status(EXIT_FAILURE)
      .run();
  }
}

#[test]
fn xdg_state_directory() {
  if let Some(state_dir) = etcetera::choose_base_strategy().unwrap().state_dir() {
    Test::new()
      .justfile("x := xdg_state_directory()")
      .args(["--evaluate", "x"])
      .stdout(state_dir.to_string_lossy())
      .run();
  } else {
    Test::new()
      .justfile("x := xdg_state_directory()")
      .args(["--evaluate", "x"])
      .stderr(
        "
          error: Call to function `xdg_state_directory` failed: state directory not found
           ——▶ justfile:1:6
            │
          1 │ x := xdg_state_directory()
            │      ^^^^^^^^^^^^^^^^^^^
        ",
      )
      .status(EXIT_FAILURE)
      .run();
  }
}
