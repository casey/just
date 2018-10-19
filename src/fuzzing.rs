use common::*;

pub fn compile(text: &str) {
  if let Err(error) = Parser::parse(text) {
    if let CompilationErrorKind::Internal{..} = error.kind {
      panic!("{}", error)
    }
  }
}
