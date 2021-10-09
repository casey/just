use crate::common::*;

use serde_json::{json, Value};

fn test(justfile: &str, value: Value) {
  let json = serde_json::to_string(&value).unwrap();
  Test::new()
    .justfile(justfile)
    .args(&["--json"])
    .stdout(format!("{}\n", json))
    .run();
}

#[test]
fn empty_justfile() {
  test(
    "",
    json!({
      "recipes": {}
    }),
  );
}

#[test]
fn single_recipe() {
  test(
    "foo:",
    json!({
      "recipes": {}
    }),
  );
}

// TODO:
// - indicate default recipe with flag
