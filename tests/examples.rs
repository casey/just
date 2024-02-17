use super::*;

#[test]
fn examples() {
  for result in fs::read_dir("examples").unwrap() {
    let entry = result.unwrap();
    let path = entry.path();
    let skipc = path.as_path().file_name().unwrap();

    if skipc != "skipc" {
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
}
