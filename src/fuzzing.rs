use common::*;

pub fn compile(text: &str) {
  Parser::parse(text).ok();
}
