use crate::common::*;

#[test]
fn examples() {
  for result in fs::read_dir("examples").unwrap() {
    let entry = result.unwrap();
    let path = entry.path();

    println!("Parsing `{}`…", path.display());

    let output = Command::new(executable_path("just"))
      .arg("--justfile")
      .arg(&path)
      .arg("--dump")
      .output()
      .unwrap();

    assert_success(&output);
  }
}
