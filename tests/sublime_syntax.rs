use crate::common::*;

#[test]
fn parse() {
  let yaml = fs::read_to_string("extras/just.sublime-syntax").unwrap();
  YamlLoader::load_from_str(&yaml).unwrap();
}
