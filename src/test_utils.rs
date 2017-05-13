extern crate glob;

use ::prelude::*;

pub fn just_binary_path() -> PathBuf {
  let exe = String::from("just") + env::consts::EXE_SUFFIX;

  let mut path = env::current_dir().unwrap();
  path.push("target");
  path.push("debug");
  path.push(&exe);

  if !path.is_file() {
    let mut pattern = env::current_dir().unwrap();
    pattern.push("target");
    pattern.push("*");
    pattern.push("debug");
    pattern.push(&exe);
    path = glob::glob(pattern.to_str().unwrap()).unwrap()
      .take_while(Result::is_ok).nth(0).unwrap().unwrap();
  }

  path
}
