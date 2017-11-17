use std::env;
use std::path::PathBuf;

pub fn just_binary_path() -> PathBuf {
  let mut path = env::current_exe().unwrap();
  path.pop();
  if path.ends_with("deps") {
    path.pop();
  }
  let exe = String::from("just") + env::consts::EXE_SUFFIX;
  path.push(exe);
  path
}

