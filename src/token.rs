use common::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'a> {
  pub index: usize,
  pub line: usize,
  pub column: usize,
  pub text: &'a str,
  pub prefix: &'a str,
  pub lexeme: &'a str,
  pub kind: TokenKind,
}

impl<'a> Token<'a> {
  pub fn error(&self, kind: CompilationErrorKind<'a>) -> CompilationError<'a> {
    CompilationError {
      column: self.column + self.prefix.len(),
      index: self.index + self.prefix.len(),
      line: self.line,
      text: self.text,
      width: Some(self.lexeme.len()),
      kind,
    }
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
  At,
  Backtick,
  Colon,
  Comma,
  Comment,
  Dedent,
  Eof,
  Eol,
  Equals,
  Indent,
  InterpolationEnd,
  InterpolationStart,
  Line,
  Name,
  ParenL,
  ParenR,
  Plus,
  RawString,
  StringToken,
  Text,
}

impl Display for TokenKind {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    use TokenKind::*;
    write!(
      f,
      "{}",
      match *self {
        Backtick => "backtick",
        Colon => "':'",
        Comma => "','",
        Comment => "comment",
        Dedent => "dedent",
        Eof => "end of file",
        Eol => "end of line",
        Equals => "'='",
        Indent => "indent",
        InterpolationEnd => "'}}'",
        InterpolationStart => "'{{'",
        Line => "command",
        Name => "name",
        Plus => "'+'",
        At => "'@'",
        ParenL => "'('",
        ParenR => "')'",
        StringToken => "string",
        RawString => "raw string",
        Text => "command text",
      }
    )
  }
}
