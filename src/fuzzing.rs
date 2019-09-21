use crate::common::*;

pub(crate) fn compile(text: &str) {
  if let Err(error) = Parser::parse(text) {
    if let CompilationErrorKind::Internal { .. } = error.kind {
      panic!("{}", error)
    }
  }
}
