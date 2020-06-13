use crate::common::*;

#[derive(Debug, PartialEq, Clone, Copy, Ord, PartialOrd, Eq)]
pub(crate) enum TokenKind {
  Asterisk,
  At,
  Backtick,
  BracketL,
  BracketR,
  Colon,
  ColonEquals,
  Comma,
  Comment,
  Dedent,
  Eof,
  Eol,
  Equals,
  Identifier,
  Indent,
  InterpolationEnd,
  InterpolationStart,
  ParenL,
  ParenR,
  Plus,
  StringCooked,
  StringRaw,
  Text,
  Unspecified,
  Whitespace,
}

impl Display for TokenKind {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    use TokenKind::*;
    write!(f, "{}", match *self {
      Asterisk => "'*'",
      At => "'@'",
      Backtick => "backtick",
      BracketL => "'['",
      BracketR => "']'",
      Colon => "':'",
      ColonEquals => "':='",
      Comma => "','",
      Comment => "comment",
      Dedent => "dedent",
      Eof => "end of file",
      Eol => "end of line",
      Equals => "'='",
      Identifier => "identifier",
      Indent => "indent",
      InterpolationEnd => "'}}'",
      InterpolationStart => "'{{'",
      ParenL => "'('",
      ParenR => "')'",
      Plus => "'+'",
      StringCooked => "cooked string",
      StringRaw => "raw string",
      Text => "command text",
      Whitespace => "whitespace",
      Unspecified => "unspecified",
    })
  }
}
