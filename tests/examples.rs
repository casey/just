use super::*;

#[test]
fn examples() {
  for result in fs::read_dir("examples").unwrap() {
    let entry = result.unwrap();
    let path = entry.path();

    println!("Parsing `{}`…", path.display());

    let output = just()
      .arg("--justfile")
      .arg(&path)
      .arg("--dump")
      .output()
      .unwrap();

    assert_success(&output);
  }
}
