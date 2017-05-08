extern crate glob;

use ::prelude::*;

pub fn just_binary_path() -> PathBuf {
  let mut binary = env::current_dir().unwrap();
  binary.push("target");
  binary.push("debug");
  binary.push("just");

  if !binary.is_file() {
    let mut pattern = env::current_dir().unwrap();
    pattern.push("target");
    pattern.push("*");
    pattern.push("debug");
    pattern.push("just");
    for path in glob::glob(pattern.to_str().unwrap()).unwrap() {
      binary = path.unwrap();
      break;
    }
  }

  binary
}
