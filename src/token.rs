use crate::common::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Token<'a> {
  pub(crate) offset: usize,
  pub(crate) length: usize,
  pub(crate) line: usize,
  pub(crate) column: usize,
  pub(crate) src: &'a str,
  pub(crate) kind: TokenKind,
}

impl<'a> Token<'a> {
  pub(crate) fn lexeme(&self) -> &'a str {
    &self.src[self.offset..self.offset + self.length]
  }

  pub(crate) fn error(&self, kind: CompilationErrorKind<'a>) -> CompilationError<'a> {
    CompilationError {
      column: self.column,
      offset: self.offset,
      line: self.line,
      src: self.src,
      width: self.length,
      kind,
    }
  }
}
