use crate::common::*;

#[derive(PartialEq, Debug)]
pub struct StringLiteral<'a> {
  pub raw: &'a str,
  pub cooked: Cow<'a, str>,
}

impl<'a> StringLiteral<'a> {
  pub fn new(token: &Token<'a>) -> CompilationResult<'a, StringLiteral<'a>> {
    let raw = &token.lexeme()[1..token.lexeme().len() - 1];

    if let TokenKind::StringRaw = token.kind {
      Ok(StringLiteral {
        cooked: Cow::Borrowed(raw),
        raw,
      })
    } else if let TokenKind::StringCooked = token.kind {
      let mut cooked = String::new();
      let mut escape = false;
      for c in raw.chars() {
        if escape {
          match c {
            'n' => cooked.push('\n'),
            'r' => cooked.push('\r'),
            't' => cooked.push('\t'),
            '\\' => cooked.push('\\'),
            '"' => cooked.push('"'),
            other => {
              return Err(
                token.error(CompilationErrorKind::InvalidEscapeSequence { character: other }),
              );
            }
          }
          escape = false;
          continue;
        }
        if c == '\\' {
          escape = true;
          continue;
        }
        cooked.push(c);
      }
      Ok(StringLiteral {
        raw,
        cooked: Cow::Owned(cooked),
      })
    } else {
      Err(token.error(CompilationErrorKind::Internal {
        message: "cook_string() called on non-string token".to_string(),
      }))
    }
  }
}

impl<'a> Display for StringLiteral<'a> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self.cooked {
      Cow::Borrowed(raw) => write!(f, "'{}'", raw),
      Cow::Owned(_) => write!(f, "\"{}\"", self.raw),
    }
  }
}
