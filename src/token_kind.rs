use crate::common::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum TokenKind {
  At,
  Backtick,
  Colon,
  ColonEquals,
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
  StringRaw,
  StringCooked,
  Text,
  Whitespace,
}

impl Display for TokenKind {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    use TokenKind::*;
    write!(
      f,
      "{}",
      match *self {
        At => "'@'",
        Backtick => "backtick",
        Colon => "':'",
        ColonEquals => "':='",
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
        ParenL => "'('",
        ParenR => "')'",
        Plus => "'+'",
        StringRaw => "raw string",
        StringCooked => "cooked string",
        Text => "command text",
        Whitespace => "whitespace",
      }
    )
  }
}
