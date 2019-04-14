use crate::common::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'a> {
  pub offset: usize,
  pub length: usize,
  pub line: usize,
  pub column: usize,
  pub text: &'a str,
  pub kind: TokenKind,
}

impl<'a> Token<'a> {
  pub fn lexeme(&self) -> &'a str {
    &self.text[self.offset..self.offset + self.length]
  }

  pub fn error(&self, kind: CompilationErrorKind<'a>) -> CompilationError<'a> {
    CompilationError {
      column: self.column,
      offset: self.offset,
      line: self.line,
      text: self.text,
      width: Some(self.length),
      kind,
    }
  }
}
