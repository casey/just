use super::*;

#[test]
fn export_recipe() {
  Test::new()
    .justfile(
      "
      export foo='bar':
        echo {{foo}}
    ",
    )
    .stdout("bar\n")
    .stderr("echo bar\n")
    .run();
}

#[test]
fn alias_recipe() {
  Test::new()
    .justfile(
      "
      alias foo='bar':
        echo {{foo}}
    ",
    )
    .stdout("bar\n")
    .stderr("echo bar\n")
    .run();
}
