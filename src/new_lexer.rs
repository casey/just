use crate::common::*;

use CompilationErrorKind::*;
use TokenKind::*;

/// Just language lexer
///
/// `self.next` points to the next character to be lexed, and
/// the text between `self.token_start` and `self.token_end` contains
/// the current token being lexed.
pub struct NewLexer<'a> {
  /// Source text
  text: &'a str,
  /// Char iterator
  chars: Chars<'a>,
  /// Tokens
  tokens: Vec<Token<'a>>,
  /// State stack
  state: Vec<State<'a>>,
  /// Current token start
  token_start: Position,
  /// Current token end
  token_end: Position,
  /// Next character
  next: Option<char>,
}

fn lex(text: &str) -> CompilationResult<Vec<Token>> {
  NewLexer::new(text).lex()
}

impl<'a> NewLexer<'a> {
  /// Create a new Lexer to lex `text`
  pub fn new(text: &'a str) -> NewLexer<'a> {
    let mut chars = text.chars();
    let next = chars.next();

    let start = Position {
      offset: 0,
      column: 0,
      line: 0,
    };

    NewLexer {
      state: vec![State::Start],
      tokens: Vec::new(),
      token_start: start,
      token_end: start,
      chars,
      next,
      text,
    }
  }

  /// Advance over the chracter in `self.next`, updating
  /// `self.token_end` accordingly.
  pub fn advance(&mut self) -> CompilationResult<'a, ()> {
    match self.next {
      Some(c) => {
        self.token_end.offset += c.len_utf8();

        match c {
          '\n' => {
            self.token_end.column = 0;
            self.token_end.line += 1;
          }
          _ => {
            self.token_end.column += 1;
          }
        }

        self.next = self.chars.next();

        Ok(())
      }
      None => Err(self.internal_error("Lexer advanced past end of text")),
    }
  }

  /// Lexeme of in-progress token
  fn lexeme(&self) -> &'a str {
    &self.text[self.token_start.offset..self.token_end.offset]
  }

  /// Un-lexed text
  fn rest(&self) -> &'a str {
    &self.text[self.token_start.offset..self.token_end.offset]
  }

  /// Length of current token
  fn current_token_length(&self) -> usize {
    self.token_end.offset - self.token_start.offset
  }

  /// Get current state
  fn state(&self) -> CompilationResult<'a, State<'a>> {
    if self.state.is_empty() {
      Err(self.internal_error("Lexer state stack empty"))
    } else {
      Ok(self.state[self.state.len() - 1])
    }
  }

  /// Pop current state from stack
  fn pop_state(&mut self) -> CompilationResult<'a, ()> {
    if self.state.pop().is_none() {
      Err(self.internal_error("Lexer attempted to pop in start state"))
    } else {
      Ok(())
    }
  }

  /// Create a new token with `kind` whose lexeme
  /// is between `self.token_start` and `self.token_end`
  pub fn token(&mut self, kind: TokenKind) {
    self.tokens.push(Token {
      offset: self.token_start.offset,
      column: self.token_start.column,
      line: self.token_start.line,
      text: self.text,
      length: self.token_end.offset - self.token_start.offset,
      kind,
    });

    // Set `token_start` to point after the lexed token
    self.token_start = self.token_end;
  }

  /// Create an internal error with `message`
  fn internal_error(&self, message: impl Into<String>) -> CompilationError<'a> {
    // Use `self.token_end` as the location of the error
    CompilationError {
      text: self.text,
      offset: self.token_end.offset,
      line: self.token_end.line,
      column: self.token_end.column,
      width: None,
      kind: CompilationErrorKind::Internal {
        message: message.into(),
      },
    }
  }

  /// Create an compilation error with `kind`
  fn error(&self, kind: CompilationErrorKind<'a>) -> CompilationError<'a> {
    // Use the in-progress token span as the location of the error
    CompilationError {
      text: self.text,
      offset: self.token_start.offset,
      line: self.token_start.line,
      column: self.token_start.column,
      width: Some(self.lexeme().len()),
      kind,
    }
  }

  /// Consume the iterator and produce a series of tokens
  pub fn lex(mut self) -> CompilationResult<'a, Vec<Token<'a>>> {
    loop {
      if self.token_start.column == 0 {
        self.lex_line_start()?;
      }

      match self.next {
        Some(first) => self.lex_token(first)?,
        None => break,
      }
    }

    Ok(self.tokens)
  }

  /// Handle blank lines and indentation
  fn lex_line_start(&mut self) -> CompilationResult<'a, ()> {
    let rest = self
      .rest()
      .chars()
      .skip_while(|&c| c == ' ' || c == '\t')
      .next();

    // Handle blank line
    if let Some('\r') | Some('\n') | None = rest {
      while let Some(' ') | Some('\t') = self.next {
        self.advance()?;
      }

      // Lex a whitespace token if the blank line was nonempty
      if self.current_token_length() > 0 {
        self.token(Whitespace);
      };

      return Ok(());
    }

    // Handle nonblank lines with no leading whitespace
    if self.next != Some(' ') && self.next != Some('\t') {
      if let State::Indent { .. } = self.state()? {
        self.token(Dedent);
        self.pop_state()?;
      }

      return Ok(());
    }

    // Handle continued indentation
    if let State::Indent { indentation } = self.state()? {
      let mut remaining = indentation.len();

      // Advance over whitespace up to length of current indentation
      while let Some(' ') | Some('\t') = self.next {
        self.advance()?;
        remaining -= 1;
        if remaining == 0 {
          break;
        }
      }

      let lexeme = self.lexeme();

      if lexeme != indentation {
        return Err(self.error(InconsistentLeadingWhitespace {
          expected: indentation,
          found: lexeme,
        }));
      }

      // Indentation matches, lex as whitespace
      self.token(Whitespace);
      return Ok(());
    }

    if self.state()? != State::Start {
      return Err(self.internal_error(format!(
        "Lexer::lex_line_start called in unexpected state: {:?}",
        self.state()
      )));
    }

    // Handle new indentation
    while let Some(' ') | Some('\t') = self.next {
      self.advance()?;
    }

    self.state.push(State::Indent {
      indentation: self.lexeme(),
    });

    self.token(Indent);

    Ok(())
  }

  /// Lex token beginning with `start`
  pub fn lex_token(&mut self, start: char) -> CompilationResult<'a, ()> {
    match start {
      'a'...'z' | 'A'...'Z' | '_' => self.lex_name(),
      _ => panic!(),
    }
  }

  /// Lex name: /[a-zA-Z_][a-zA-Z0-9_]*/
  pub fn lex_name(&mut self) -> CompilationResult<'a, ()> {
    while let Some('a'...'z') | Some('A'...'Z') | Some('0'...'9') | Some('_') = self.next {
      self.advance()?;
    }

    self.token(Name);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! summary_test {
    ($name:ident, $input:expr, $expected:expr $(,)*) => {
      #[test]
      fn $name() {
        let input = $input;
        let expected = $expected;
        let tokens = crate::new_lexer::lex(input).unwrap();
        let roundtrip = tokens
          .iter()
          .map(Token::lexeme)
          .collect::<Vec<&str>>()
          .join("");
        let actual = token_summary(&tokens);
        if actual != expected {
          panic!(
            "token summary mismatch:\nexpected: {}\ngot:      {}\n",
            expected, actual
          );
        }
        assert_eq!(input, roundtrip);
      }
    };
  }

  fn token_summary(tokens: &[Token]) -> String {
    tokens
      .iter()
      .map(|t| match t.kind {
        At => "@",
        Backtick => "`",
        Colon => ":",
        Comma => ",",
        Comment { .. } => "#",
        Dedent => "<",
        Eof => ".",
        Eol => "$",
        Equals => "=",
        Indent { .. } => ">",
        InterpolationEnd => "}",
        InterpolationStart => "{",
        Line { .. } => "^",
        Name => "N",
        ParenL => "(",
        ParenR => ")",
        Plus => "+",
        RawString => "'",
        StringToken => "\"",
        Text => "_",
        Whitespace => "",
      })
      .collect::<Vec<&str>>()
      .join("")
  }

  macro_rules! error_test {
    (
      name:     $name:ident,
      input:    $input:expr,
      offset:   $offset:expr,
      line:     $line:expr,
      column:   $column:expr,
      width:    $width:expr,
      kind:     $kind:expr,
    ) => {
      #[test]
      fn $name() {
        let input = $input;

        let expected = CompilationError {
          text: input,
          offset: $offset,
          line: $line,
          column: $column,
          width: $width,
          kind: $kind,
        };

        if let Err(error) = NewLexer::lex(input) {
          assert_eq!(error.text, expected.text);
          assert_eq!(error.offset, expected.offset);
          assert_eq!(error.line, expected.line);
          assert_eq!(error.column, expected.column);
          assert_eq!(error.kind, expected.kind);
          assert_eq!(error, expected);
        } else {
          panic!("tokenize succeeded but expected: {}\n{}", expected, input);
        }
      }
    };
  }

  summary_test! {
    name,
    "foo",
    "N",
  }

  summary_test! {
    indent,
    "   foo",
    ">N",
  }
}
