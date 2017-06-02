extern crate glob;

use ::prelude::*;

pub fn just_binary_path() -> PathBuf {
  let mut path = env::current_exe().unwrap();
  path.pop();
  if path.ends_with("deps") {
    path.pop();
  }
  path.push("just");
  path
}
