use super::*;

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
  Test::new()
    .justfile("x := executable_directory()")
    .args(["--evaluate", "x"])
    .stdout(dirs::executable_dir().unwrap_or_default().to_string_lossy())
    .run();
}

#[test]
fn home_directory() {
  Test::new()
    .justfile("x := home_directory()")
    .args(["--evaluate", "x"])
    .stdout(dirs::home_dir().unwrap_or_default().to_string_lossy())
    .run();
}
