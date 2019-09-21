use crate::common::*;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Token<'a> {
  pub(crate) offset: usize,
  pub(crate) length: usize,
  pub(crate) line: usize,
  pub(crate) column: usize,
  pub(crate) text: &'a str,
  pub(crate) kind: TokenKind,
}

impl<'a> Token<'a> {
  pub(crate) fn lexeme(&self) -> &'a str {
    &self.text[self.offset..self.offset + self.length]
  }

  pub(crate) fn error(&self, kind: CompilationErrorKind<'a>) -> CompilationError<'a> {
    CompilationError {
      column: self.column,
      offset: self.offset,
      line: self.line,
      text: self.text,
      width: self.length,
      kind,
    }
  }
}
