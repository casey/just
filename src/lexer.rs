use crate::common::*;

use CompilationErrorKind::*;
use TokenKind::*;

/// Just language lexer
///
/// The lexer proceeds character-by-character, as opposed to using
/// regular expressions to lex tokens or semi-tokens at a time. As a
/// result, it is verbose and straightforward. Just used to have a
/// regex-based lexer, which was slower and generally godawful. However,
/// this should not be taken as a slight against regular expressions,
/// the lexer was just idiosyncratically bad.
pub(crate) struct Lexer<'src> {
  /// Source text
  src: &'src str,
  /// Char iterator
  chars: Chars<'src>,
  /// Tokens
  tokens: Vec<Token<'src>>,
  /// State stack
  state: Vec<State<'src>>,
  /// Current token start
  token_start: Position,
  /// Current token end
  token_end: Position,
  /// Next character to be lexed
  next: Option<char>,
}

impl<'src> Lexer<'src> {
  /// Lex `text`
  pub(crate) fn lex(src: &str) -> CompilationResult<Vec<Token>> {
    Lexer::new(src).tokenize()
  }

  /// Create a new Lexer to lex `text`
  fn new(src: &'src str) -> Lexer<'src> {
    let mut chars = src.chars();
    let next = chars.next();

    let start = Position {
      offset: 0,
      column: 0,
      line: 0,
    };

    Lexer {
      state: vec![State::Normal],
      tokens: Vec::new(),
      token_start: start,
      token_end: start,
      chars,
      next,
      src,
    }
  }

  /// Advance over the chracter in `self.next`, updating
  /// `self.token_end` accordingly.
  fn advance(&mut self) -> CompilationResult<'src, ()> {
    match self.next {
      Some(c) => {
        let len_utf8 = c.len_utf8();

        self.token_end.offset += len_utf8;

        match c {
          '\n' => {
            self.token_end.column = 0;
            self.token_end.line += 1;
          }
          _ => {
            self.token_end.column += len_utf8;
          }
        }

        self.next = self.chars.next();

        Ok(())
      }
      None => Err(self.internal_error("Lexer advanced past end of text")),
    }
  }

  /// Lexeme of in-progress token
  fn lexeme(&self) -> &'src str {
    &self.src[self.token_start.offset..self.token_end.offset]
  }

  /// Length of current token
  fn current_token_length(&self) -> usize {
    self.token_end.offset - self.token_start.offset
  }

  /// Is next character c?
  fn next_is(&self, c: char) -> bool {
    self.next == Some(c)
  }

  /// Is next character ' ' or '\t'?
  fn next_is_whitespace(&self) -> bool {
    self.next_is(' ') || self.next_is('\t')
  }

  /// Un-lexed text
  fn rest(&self) -> &'src str {
    &self.src[self.token_end.offset..]
  }

  /// Check if unlexed text begins with prefix
  fn rest_starts_with(&self, prefix: &str) -> bool {
    self.rest().starts_with(prefix)
  }

  /// Does rest start with "\n" or "\r\n"?
  fn at_eol(&self) -> bool {
    self.next_is('\n') || self.rest_starts_with("\r\n")
  }

  /// Are we at end-of-line or end-of-file?
  fn at_eol_or_eof(&self) -> bool {
    self.at_eol() || self.rest().is_empty()
  }

  /// Get current state
  fn state(&self) -> CompilationResult<'src, State<'src>> {
    if self.state.is_empty() {
      Err(self.internal_error("Lexer state stack empty"))
    } else {
      Ok(self.state[self.state.len() - 1])
    }
  }

  /// Pop current state from stack
  fn pop_state(&mut self) -> CompilationResult<'src, ()> {
    if self.state.pop().is_none() {
      Err(self.internal_error("Lexer attempted to pop in start state"))
    } else {
      Ok(())
    }
  }

  /// Create a new token with `kind` whose lexeme
  /// is between `self.token_start` and `self.token_end`
  fn token(&mut self, kind: TokenKind) {
    self.tokens.push(Token {
      offset: self.token_start.offset,
      column: self.token_start.column,
      line: self.token_start.line,
      src: self.src,
      length: self.token_end.offset - self.token_start.offset,
      kind,
    });

    // Set `token_start` to point after the lexed token
    self.token_start = self.token_end;
  }

  /// Create an internal error with `message`
  fn internal_error(&self, message: impl Into<String>) -> CompilationError<'src> {
    // Use `self.token_end` as the location of the error
    let token = Token {
      src: self.src,
      offset: self.token_end.offset,
      line: self.token_end.line,
      column: self.token_end.column,
      length: 0,
      kind: Unspecified,
    };
    CompilationError {
      kind: CompilationErrorKind::Internal {
        message: message.into(),
      },
      token,
    }
  }

  /// Create a compilation error with `kind`
  fn error(&self, kind: CompilationErrorKind<'src>) -> CompilationError<'src> {
    // Use the in-progress token span as the location of the error.

    // The width of the error site to highlight depends on the kind of error:
    let length = match kind {
      // highlight ' or "
      UnterminatedString => 1,
      // highlight `
      UnterminatedBacktick => 1,
      // highlight the full token
      _ => self.lexeme().len(),
    };

    let token = Token {
      kind: Unspecified,
      src: self.src,
      offset: self.token_start.offset,
      line: self.token_start.line,
      column: self.token_start.column,
      length,
    };

    CompilationError { token, kind }
  }

  fn unterminated_interpolation_error(
    &self,
    interpolation_start: Token<'src>,
  ) -> CompilationError<'src> {
    CompilationError {
      token: interpolation_start,
      kind: UnterminatedInterpolation,
    }
  }

  /// True if `text` could be an identifier
  pub(crate) fn is_identifier(text: &str) -> bool {
    if !text
      .chars()
      .next()
      .map(Self::is_identifier_start)
      .unwrap_or(false)
    {
      return false;
    }

    for c in text.chars().skip(1) {
      if !Self::is_identifier_continue(c) {
        return false;
      }
    }

    true
  }

  /// True if `c` can be the first character of an identifier
  fn is_identifier_start(c: char) -> bool {
    match c {
      'a'..='z' | 'A'..='Z' | '_' => true,
      _ => false,
    }
  }

  /// True if `c` can be a continuation character of an idenitifier
  fn is_identifier_continue(c: char) -> bool {
    if Self::is_identifier_start(c) {
      return true;
    }

    match c {
      '0'..='9' | '-' => true,
      _ => false,
    }
  }

  /// Consume the text and produce a series of tokens
  fn tokenize(mut self) -> CompilationResult<'src, Vec<Token<'src>>> {
    loop {
      if self.token_start.column == 0 {
        self.lex_line_start()?;
      }

      match self.next {
        Some(first) => match self.state()? {
          State::Normal => self.lex_normal(first)?,
          State::Interpolation {
            interpolation_start,
          } => self.lex_interpolation(interpolation_start, first)?,
          State::Text => self.lex_text()?,
          State::Indented { .. } => self.lex_indented()?,
        },
        None => break,
      }
    }

    if let State::Interpolation {
      interpolation_start,
    } = self.state()?
    {
      return Err(self.unterminated_interpolation_error(interpolation_start));
    }

    if let State::Indented { .. } | State::Text = self.state()? {
      self.token(Dedent);
    }

    self.token(Eof);

    assert_eq!(self.token_start.offset, self.token_end.offset);
    assert_eq!(self.token_start.offset, self.src.len());

    Ok(self.tokens)
  }

  /// Handle blank lines and indentation
  fn lex_line_start(&mut self) -> CompilationResult<'src, ()> {
    let nonblank_index = self
      .rest()
      .char_indices()
      .skip_while(|&(_, c)| c == ' ' || c == '\t')
      .map(|(i, _)| i)
      .next()
      .unwrap_or_else(|| self.rest().len());

    let rest = &self.rest()[nonblank_index..];

    // Handle blank line
    if rest.starts_with('\n') || rest.starts_with("\r\n") || rest.is_empty() {
      while self.next_is_whitespace() {
        self.advance()?;
      }

      // Lex a whitespace token if the blank line was nonempty
      if self.current_token_length() > 0 {
        self.token(Whitespace);
      };

      return Ok(());
    }

    // Handle nonblank lines with no leading whitespace
    if !self.next_is_whitespace() {
      if let State::Indented { .. } = self.state()? {
        self.token(Dedent);
        self.pop_state()?;
      }

      return Ok(());
    }

    // Handle continued indentation
    if let State::Indented { indentation } = self.state()? {
      if self.rest_starts_with(indentation) {
        for _ in indentation.chars() {
          self.advance()?;
        }

        // Indentation matches, lex as whitespace
        self.token(Whitespace);

        return Ok(());
      }

      // Consume whitespace characters, matching or not, up to the length
      // of expected indentation
      for _ in indentation.chars().zip(self.rest().chars()) {
        if self.next_is_whitespace() {
          self.advance()?;
        } else {
          break;
        }
      }

      // We've either advanced over not enough whitespace or mismatching
      // whitespace, so return an error
      return Err(self.error(InconsistentLeadingWhitespace {
        expected: indentation,
        found: self.lexeme(),
      }));
    }

    if self.state()? != State::Normal {
      return Err(self.internal_error(format!(
        "Lexer::lex_line_start called in unexpected state: {:?}",
        self.state()
      )));
    }

    // Handle new indentation
    while self.next_is_whitespace() {
      self.advance()?;
    }

    let indentation = self.lexeme();

    let spaces = indentation.chars().any(|c| c == ' ');
    let tabs = indentation.chars().any(|c| c == '\t');

    if spaces && tabs {
      return Err(self.error(MixedLeadingWhitespace {
        whitespace: indentation,
      }));
    }

    self.state.push(State::Indented { indentation });

    self.token(Indent);

    Ok(())
  }

  /// Lex token beginning with `start` in normal state
  fn lex_normal(&mut self, start: char) -> CompilationResult<'src, ()> {
    match start {
      '@' => self.lex_single(At),
      '[' => self.lex_single(BracketL),
      ']' => self.lex_single(BracketR),
      '=' => self.lex_single(Equals),
      ',' => self.lex_single(Comma),
      ':' => self.lex_colon(),
      '(' => self.lex_single(ParenL),
      ')' => self.lex_single(ParenR),
      '{' => self.lex_brace_l(),
      '}' => self.lex_brace_r(),
      '+' => self.lex_single(Plus),
      '\n' => self.lex_single(Eol),
      '\r' => self.lex_cr_lf(),
      '#' => self.lex_comment(),
      '`' => self.lex_backtick(),
      ' ' | '\t' => self.lex_whitespace(),
      '\'' => self.lex_raw_string(),
      '"' => self.lex_cooked_string(),
      _ => {
        if Self::is_identifier_start(start) {
          self.lex_identifier()
        } else {
          self.advance()?;
          Err(self.error(UnknownStartOfToken))
        }
      }
    }
  }

  /// Lex token beginning with `start` in interpolation state
  fn lex_interpolation(
    &mut self,
    interpolation_start: Token<'src>,
    start: char,
  ) -> CompilationResult<'src, ()> {
    // Check for end of interpolation
    if self.rest_starts_with("}}") {
      // Pop interpolation state
      self.pop_state()?;
      // Emit interpolation end token
      self.lex_double(InterpolationEnd)
    } else if self.at_eol_or_eof() {
      // Return unterminated interpolation error that highlights the opening {{
      Err(self.unterminated_interpolation_error(interpolation_start))
    } else {
      // Otherwise lex as if we are in normal state
      self.lex_normal(start)
    }
  }

  /// Lex token beginning with `start` in text state
  fn lex_text(&mut self) -> CompilationResult<'src, ()> {
    enum Terminator {
      Newline,
      NewlineCarriageReturn,
      Interpolation,
      EndOfFile,
    }

    use Terminator::*;

    let terminator = loop {
      if let Some('\n') = self.next {
        break Newline;
      }

      if self.rest_starts_with("\r\n") {
        break NewlineCarriageReturn;
      }

      if self.rest_starts_with("{{") {
        break Interpolation;
      }

      if self.next.is_none() {
        break EndOfFile;
      }

      self.advance()?;
    };

    // emit text token containing text so far
    if self.current_token_length() > 0 {
      self.token(Text);
    }

    match terminator {
      Newline => {
        self.state.pop();
        self.lex_single(Eol)
      }
      NewlineCarriageReturn => {
        self.state.pop();
        self.lex_double(Eol)
      }
      Interpolation => {
        self.lex_double(InterpolationStart)?;
        self.state.push(State::Interpolation {
          interpolation_start: self.tokens[self.tokens.len() - 1],
        });
        Ok(())
      }
      EndOfFile => self.pop_state(),
    }
  }

  /// Lex token beginning with `start` in indented state
  fn lex_indented(&mut self) -> CompilationResult<'src, ()> {
    self.state.push(State::Text);
    Ok(())
  }

  /// Lex a single character token
  fn lex_single(&mut self, kind: TokenKind) -> CompilationResult<'src, ()> {
    self.advance()?;
    self.token(kind);
    Ok(())
  }

  /// Lex a double character token
  fn lex_double(&mut self, kind: TokenKind) -> CompilationResult<'src, ()> {
    self.advance()?;
    self.advance()?;
    self.token(kind);
    Ok(())
  }

  /// Lex a token starting with ':'
  fn lex_colon(&mut self) -> CompilationResult<'src, ()> {
    self.advance()?;

    if self.next_is('=') {
      self.advance()?;
      self.token(ColonEquals);
    } else {
      self.token(Colon);
    }

    Ok(())
  }

  /// Lex a token starting with '{'
  fn lex_brace_l(&mut self) -> CompilationResult<'src, ()> {
    if !self.rest_starts_with("{{") {
      self.advance()?;

      return Err(self.error(UnknownStartOfToken));
    }

    self.lex_double(InterpolationStart)
  }

  /// Lex a token starting with '}'
  fn lex_brace_r(&mut self) -> CompilationResult<'src, ()> {
    if !self.rest_starts_with("}}") {
      self.advance()?;

      return Err(self.error(UnknownStartOfToken));
    }

    self.lex_double(InterpolationEnd)
  }

  /// Lex a carriage return and line feed
  fn lex_cr_lf(&mut self) -> CompilationResult<'src, ()> {
    if !self.rest_starts_with("\r\n") {
      // advance over \r
      self.advance()?;

      return Err(self.error(UnpairedCarriageReturn));
    }

    self.lex_double(Eol)
  }

  /// Lex name: [a-zA-Z_][a-zA-Z0-9_]*
  fn lex_identifier(&mut self) -> CompilationResult<'src, ()> {
    // advance over initial character
    self.advance()?;

    while let Some(c) = self.next {
      if !Self::is_identifier_continue(c) {
        break;
      }

      self.advance()?;
    }

    self.token(Identifier);

    Ok(())
  }

  /// Lex comment: #[^\r\n]
  fn lex_comment(&mut self) -> CompilationResult<'src, ()> {
    // advance over #
    self.advance()?;

    while !self.at_eol_or_eof() {
      self.advance()?;
    }

    self.token(Comment);

    Ok(())
  }

  /// Lex backtick: `[^\r\n]*`
  fn lex_backtick(&mut self) -> CompilationResult<'src, ()> {
    // advance over initial `
    self.advance()?;

    while !self.next_is('`') {
      if self.at_eol_or_eof() {
        return Err(self.error(UnterminatedBacktick));
      }

      self.advance()?;
    }

    self.advance()?;
    self.token(Backtick);

    Ok(())
  }

  /// Lex whitespace: [ \t]+
  fn lex_whitespace(&mut self) -> CompilationResult<'src, ()> {
    while self.next_is_whitespace() {
      self.advance()?
    }

    self.token(Whitespace);

    Ok(())
  }

  /// Lex raw string: '[^']*'
  fn lex_raw_string(&mut self) -> CompilationResult<'src, ()> {
    // advance over opening '
    self.advance()?;

    loop {
      match self.next {
        Some('\'') => break,
        None => return Err(self.error(UnterminatedString)),
        _ => {}
      }

      self.advance()?;
    }

    // advance over closing '
    self.advance()?;

    self.token(StringRaw);

    Ok(())
  }

  /// Lex cooked string: "[^"\n\r]*" (also processes escape sequences)
  fn lex_cooked_string(&mut self) -> CompilationResult<'src, ()> {
    // advance over opening "
    self.advance()?;

    let mut escape = false;

    loop {
      match self.next {
        Some('\r') | Some('\n') | None => return Err(self.error(UnterminatedString)),
        Some('"') if !escape => break,
        Some('\\') if !escape => escape = true,
        _ => escape = false,
      }

      self.advance()?;
    }

    // advance over closing "
    self.advance()?;

    self.token(StringCooked);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use pretty_assertions::assert_eq;

  macro_rules! test {
    {
      name:   $name:ident,
      text:   $text:expr,
      tokens: ($($kind:ident $(: $lexeme:literal)?),* $(,)?)$(,)?
    } => {
      #[test]
      fn $name() {
        let kinds: &[TokenKind] = &[$($kind,)* Eof];

        let lexemes: &[&str] = &[$(lexeme!($kind $(, $lexeme)?),)* ""];

        test($text, kinds, lexemes);
      }
    }
  }

  macro_rules! lexeme {
    {
      $kind:ident, $lexeme:literal
    } => {
      $lexeme
    };
    {
      $kind:ident
    } => {
      default_lexeme($kind)
    }
  }

  fn test(text: &str, want_kinds: &[TokenKind], want_lexemes: &[&str]) {
    let text = testing::unindent(text);

    let have = Lexer::lex(&text).unwrap();

    let have_kinds = have
      .iter()
      .map(|token| token.kind)
      .collect::<Vec<TokenKind>>();

    let have_lexemes = have
      .iter()
      .map(|token| token.lexeme())
      .collect::<Vec<&str>>();

    assert_eq!(have_kinds, want_kinds, "Token kind mismatch");
    assert_eq!(have_lexemes, want_lexemes, "Token lexeme mismatch");

    let mut roundtrip = String::new();

    for lexeme in have_lexemes {
      roundtrip.push_str(lexeme);
    }

    assert_eq!(roundtrip, text, "Roundtrip mismatch");

    let mut offset = 0;
    let mut line = 0;
    let mut column = 0;

    for token in have {
      assert_eq!(token.offset, offset);
      assert_eq!(token.line, line);
      assert_eq!(token.lexeme().len(), token.length);
      assert_eq!(token.column, column);

      for c in token.lexeme().chars() {
        if c == '\n' {
          line += 1;
          column = 0;
        } else {
          column += c.len_utf8();
        }
      }

      offset += token.length;
    }
  }

  fn default_lexeme(kind: TokenKind) -> &'static str {
    match kind {
      // Fixed lexemes
      At => "@",
      BracketL => "[",
      BracketR => "]",
      Colon => ":",
      ColonEquals => ":=",
      Comma => ",",
      Eol => "\n",
      Equals => "=",
      Indent => "  ",
      InterpolationEnd => "}}",
      InterpolationStart => "{{",
      ParenL => "(",
      ParenR => ")",
      Plus => "+",
      Whitespace => " ",

      // Empty lexemes
      Dedent | Eof => "",

      // Variable lexemes
      Text | StringCooked | StringRaw | Identifier | Comment | Backtick | Unspecified => {
        panic!("Token {:?} has no default lexeme", kind)
      }
    }
  }

  macro_rules! error {
    (
      name:   $name:ident,
      input:  $input:expr,
      offset: $offset:expr,
      line:   $line:expr,
      column: $column:expr,
      width:  $width:expr,
      kind:   $kind:expr,
    ) => {
      #[test]
      fn $name() {
        error($input, $offset, $line, $column, $width, $kind);
      }
    };
  }

  fn error(
    src: &str,
    offset: usize,
    line: usize,
    column: usize,
    length: usize,
    kind: CompilationErrorKind,
  ) {
    match Lexer::lex(src) {
      Ok(_) => panic!("Lexing succeeded but expected"),
      Err(have) => {
        let want = CompilationError {
          token: Token {
            kind: have.token.kind,
            src,
            offset,
            line,
            column,
            length,
          },
          kind,
        };
        assert_eq!(have, want);
      }
    }
  }

  test! {
    name:   name_new,
    text:   "foo",
    tokens: (Identifier:"foo"),
  }

  test! {
    name:   comment,
    text:   "# hello",
    tokens: (Comment:"# hello"),
  }

  test! {
    name:   backtick,
    text:   "`echo`",
    tokens: (Backtick:"`echo`"),
  }

  test! {
    name:   raw_string,
    text:   "'hello'",
    tokens: (StringRaw:"'hello'"),
  }

  test! {
    name:   cooked_string,
    text:   "\"hello\"",
    tokens: (StringCooked:"\"hello\""),
  }

  test! {
    name:   export_concatination,
    text:   "export foo = 'foo' + 'bar'",
    tokens: (
      Identifier:"export",
      Whitespace,
      Identifier:"foo",
      Whitespace,
      Equals,
      Whitespace,
      StringRaw:"'foo'",
      Whitespace,
      Plus,
      Whitespace,
      StringRaw:"'bar'",
    )
  }

  test! {
    name: export_complex,
    text: "export foo = ('foo' + 'bar') + `baz`",
    tokens: (
      Identifier:"export",
      Whitespace,
      Identifier:"foo",
      Whitespace,
      Equals,
      Whitespace,
      ParenL,
      StringRaw:"'foo'",
      Whitespace,
      Plus,
      Whitespace,
      StringRaw:"'bar'",
      ParenR,
      Whitespace,
      Plus,
      Whitespace,
      Backtick:"`baz`",
    ),
  }

  test! {
    name:   eol_linefeed,
    text:   "\n",
    tokens: (Eol),
  }

  test! {
    name:   eol_carriage_return_linefeed,
    text:   "\r\n",
    tokens: (Eol:"\r\n"),
  }

  test! {
    name:   indented_line,
    text:   "foo:\n a",
    tokens: (Identifier:"foo", Colon, Eol, Indent:" ", Text:"a", Dedent),
  }

  test! {
    name: indented_block,
    text: "
      foo:
        a
        b
        c
    ",
    tokens: (
      Identifier:"foo",
      Colon,
      Eol,
      Indent,
      Text:"a",
      Eol,
      Whitespace:"  ",
      Text:"b",
      Eol,
      Whitespace:"  ",
      Text:"c",
      Eol,
      Dedent,
    )
  }

  test! {
    name: indented_block_followed_by_item,
    text: "
      foo:
        a
      b:
    ",
    tokens: (
      Identifier:"foo",
      Colon,
      Eol,
      Indent,
      Text:"a",
      Eol,
      Dedent,
      Identifier:"b",
      Colon,
      Eol,
    )
  }

  test! {
    name: indented_block_followed_by_blank,
    text: "
      foo:
          a

      b:
    ",
    tokens: (
      Identifier:"foo",
      Colon,
      Eol,
      Indent:"    ",
      Text:"a",
      Eol,
      Eol,
      Dedent,
      Identifier:"b",
      Colon,
      Eol,
    ),
  }

  test! {
    name: indented_line_containing_unpaired_carriage_return,
    text: "foo:\n \r \n",
    tokens: (
      Identifier:"foo",
      Colon,
      Eol,
      Indent:" ",
      Text:"\r ",
      Eol,
      Dedent,
    ),
  }

  test! {
    name: indented_blocks,
    text: "
      b: a
        @mv a b

      a:
        @touch F
        @touch a

      d: c
        @rm c

      c: b
        @mv b c
    ",
    tokens: (
      Identifier:"b",
      Colon,
      Whitespace,
      Identifier:"a",
      Eol,
      Indent,
      Text:"@mv a b",
      Eol,
      Eol,
      Dedent,
      Identifier:"a",
      Colon,
      Eol,
      Indent,
      Text:"@touch F",
      Eol,
      Whitespace:"  ",
      Text:"@touch a",
      Eol,
      Eol,
      Dedent,
      Identifier:"d",
      Colon,
      Whitespace,
      Identifier:"c",
      Eol,
      Indent,
      Text:"@rm c",
      Eol,
      Eol,
      Dedent,
      Identifier:"c",
      Colon,
      Whitespace,
      Identifier:"b",
      Eol,
      Indent,
      Text:"@mv b c",
      Eol,
      Dedent
    ),
  }

  test! {
    name: interpolation_empty,
    text: "hello:\n echo {{}}",
    tokens: (
      Identifier:"hello",
      Colon,
      Eol,
      Indent:" ",
      Text:"echo ",
      InterpolationStart,
      InterpolationEnd,
      Dedent,
    ),
  }

  test! {
    name: interpolation_expression,
    text: "hello:\n echo {{`echo hello` + `echo goodbye`}}",
    tokens: (
      Identifier:"hello",
      Colon,
      Eol,
      Indent:" ",
      Text:"echo ",
      InterpolationStart,
      Backtick:"`echo hello`",
      Whitespace,
      Plus,
      Whitespace,
      Backtick:"`echo goodbye`",
      InterpolationEnd,
      Dedent,
    ),
  }

  test! {
    name: tokenize_names,
    text: "
      foo
      bar-bob
      b-bob_asdfAAAA
      test123
    ",
    tokens: (
      Identifier:"foo",
      Eol,
      Identifier:"bar-bob",
      Eol,
      Identifier:"b-bob_asdfAAAA",
      Eol,
      Identifier:"test123",
      Eol,
    ),
  }

  test! {
    name: tokenize_indented_line,
    text: "foo:\n a",
    tokens: (
      Identifier:"foo",
      Colon,
      Eol,
      Indent:" ",
      Text:"a",
      Dedent,
    ),
  }

  test! {
    name: tokenize_indented_block,
    text: "
      foo:
        a
        b
        c
    ",
    tokens: (
      Identifier:"foo",
      Colon,
      Eol,
      Indent,
      Text:"a",
      Eol,
      Whitespace:"  ",
      Text:"b",
      Eol,
      Whitespace:"  ",
      Text:"c",
      Eol,
      Dedent,
    ),
  }

  test! {
    name: tokenize_strings,
    text: r#"a = "'a'" + '"b"' + "'c'" + '"d"'#echo hello"#,
    tokens: (
      Identifier:"a",
      Whitespace,
      Equals,
      Whitespace,
      StringCooked:"\"'a'\"",
      Whitespace,
      Plus,
      Whitespace,
      StringRaw:"'\"b\"'",
      Whitespace,
      Plus,
      Whitespace,
      StringCooked:"\"'c'\"",
      Whitespace,
      Plus,
      Whitespace,
      StringRaw:"'\"d\"'",
      Comment:"#echo hello",
    )
  }

  test! {
    name: tokenize_recipe_interpolation_eol,
    text: "
      foo: # some comment
       {{hello}}
    ",
    tokens: (
      Identifier:"foo",
      Colon,
      Whitespace,
      Comment:"# some comment",
      Eol,
      Indent:" ",
      InterpolationStart,
      Identifier:"hello",
      InterpolationEnd,
      Eol,
      Dedent
    ),
  }

  test! {
    name: tokenize_recipe_interpolation_eof,
    text: "foo: # more comments
 {{hello}}
# another comment
",
    tokens: (
      Identifier:"foo",
      Colon,
      Whitespace,
      Comment:"# more comments",
      Eol,
      Indent:" ",
      InterpolationStart,
      Identifier:"hello",
      InterpolationEnd,
      Eol,
      Dedent,
      Comment:"# another comment",
      Eol,
    ),
  }

  test! {
    name: tokenize_recipe_complex_interpolation_expression,
    text: "foo: #lol\n {{a + b + \"z\" + blarg}}",
    tokens: (
      Identifier:"foo",
      Colon,
      Whitespace:" ",
      Comment:"#lol",
      Eol,
      Indent:" ",
      InterpolationStart,
      Identifier:"a",
      Whitespace,
      Plus,
      Whitespace,
      Identifier:"b",
      Whitespace,
      Plus,
      Whitespace,
      StringCooked:"\"z\"",
      Whitespace,
      Plus,
      Whitespace,
      Identifier:"blarg",
      InterpolationEnd,
      Dedent,
    ),
  }

  test! {
    name: tokenize_recipe_multiple_interpolations,
    text: "foo:,#ok\n {{a}}0{{b}}1{{c}}",
    tokens: (
      Identifier:"foo",
      Colon,
      Comma,
      Comment:"#ok",
      Eol,
      Indent:" ",
      InterpolationStart,
      Identifier:"a",
      InterpolationEnd,
      Text:"0",
      InterpolationStart,
      Identifier:"b",
      InterpolationEnd,
      Text:"1",
      InterpolationStart,
      Identifier:"c",
      InterpolationEnd,
      Dedent,

    ),
  }

  test! {
    name: tokenize_junk,
    text: "
      bob

      hello blah blah blah : a b c #whatever
    ",
    tokens: (
      Identifier:"bob",
      Eol,
      Eol,
      Identifier:"hello",
      Whitespace,
      Identifier:"blah",
      Whitespace,
      Identifier:"blah",
      Whitespace,
      Identifier:"blah",
      Whitespace,
      Colon,
      Whitespace,
      Identifier:"a",
      Whitespace,
      Identifier:"b",
      Whitespace,
      Identifier:"c",
      Whitespace,
      Comment:"#whatever",
      Eol,
    )
  }

  test! {
    name: tokenize_empty_lines,
    text: "

      # this does something
      hello:
        asdf
        bsdf

        csdf

        dsdf # whatever

      # yolo
    ",
    tokens: (
      Eol,
      Comment:"# this does something",
      Eol,
      Identifier:"hello",
      Colon,
      Eol,
      Indent,
      Text:"asdf",
      Eol,
      Whitespace:"  ",
      Text:"bsdf",
      Eol,
      Eol,
      Whitespace:"  ",
      Text:"csdf",
      Eol,
      Eol,
      Whitespace:"  ",
      Text:"dsdf # whatever",
      Eol,
      Eol,
      Dedent,
      Comment:"# yolo",
      Eol,
    ),
  }

  test! {
    name: tokenize_comment_before_variable,
    text: "
      #
      A='1'
      echo:
        echo {{A}}
    ",
    tokens: (
      Comment:"#",
      Eol,
      Identifier:"A",
      Equals,
      StringRaw:"'1'",
      Eol,
      Identifier:"echo",
      Colon,
      Eol,
      Indent,
      Text:"echo ",
      InterpolationStart,
      Identifier:"A",
      InterpolationEnd,
      Eol,
      Dedent,
    ),
  }

  test! {
    name: tokenize_interpolation_backticks,
    text: "hello:\n echo {{`echo hello` + `echo goodbye`}}",
    tokens: (
      Identifier:"hello",
      Colon,
      Eol,
      Indent:" ",
      Text:"echo ",
      InterpolationStart,
      Backtick:"`echo hello`",
      Whitespace,
      Plus,
      Whitespace,
      Backtick:"`echo goodbye`",
      InterpolationEnd,
      Dedent
    ),
  }

  test! {
    name: tokenize_empty_interpolation,
    text: "hello:\n echo {{}}",
    tokens: (
      Identifier:"hello",
      Colon,
      Eol,
      Indent:" ",
      Text:"echo ",
      InterpolationStart,
      InterpolationEnd,
      Dedent,
    ),
  }

  test! {
    name: tokenize_assignment_backticks,
    text: "a = `echo hello` + `echo goodbye`",
    tokens: (
      Identifier:"a",
      Whitespace,
      Equals,
      Whitespace,
      Backtick:"`echo hello`",
      Whitespace,
      Plus,
      Whitespace,
      Backtick:"`echo goodbye`",
    ),
  }

  test! {
    name: tokenize_multiple,
    text: "

      hello:
        a
        b

        c

        d

      # hello
      bob:
        frank
       \t
    ",
    tokens: (
      Eol,
      Identifier:"hello",
      Colon,
      Eol,
      Indent,
      Text:"a",
      Eol,
      Whitespace:"  ",
      Text:"b",
      Eol,
      Eol,
      Whitespace:"  ",
      Text:"c",
      Eol,
      Eol,
      Whitespace:"  ",
      Text:"d",
      Eol,
      Eol,
      Dedent,
      Comment:"# hello",
      Eol,
      Identifier:"bob",
      Colon,
      Eol,
      Indent:"  ",
      Text:"frank",
      Eol,
      Eol,
      Dedent,
    ),
  }

  test! {
    name: tokenize_comment,
    text: "a:=#",
    tokens: (
      Identifier:"a",
      ColonEquals,
      Comment:"#",
    ),
  }

  test! {
    name: tokenize_comment_with_bang,
    text: "a:=#foo!",
    tokens: (
      Identifier:"a",
      ColonEquals,
      Comment:"#foo!",
    ),
  }

  test! {
    name: tokenize_order,
    text: "
      b: a
        @mv a b

      a:
        @touch F
        @touch a

      d: c
        @rm c

      c: b
        @mv b c
    ",
    tokens: (
      Identifier:"b",
      Colon,
      Whitespace,
      Identifier:"a",
      Eol,
      Indent,
      Text:"@mv a b",
      Eol,
      Eol,
      Dedent,
      Identifier:"a",
      Colon,
      Eol,
      Indent,
      Text:"@touch F",
      Eol,
      Whitespace:"  ",
      Text:"@touch a",
      Eol,
      Eol,
      Dedent,
      Identifier:"d",
      Colon,
      Whitespace,
      Identifier:"c",
      Eol,
      Indent,
      Text:"@rm c",
      Eol,
      Eol,
      Dedent,
      Identifier:"c",
      Colon,
      Whitespace,
      Identifier:"b",
      Eol,
      Indent,
      Text:"@mv b c",
      Eol,
      Dedent,
    ),
  }

  test! {
    name: tokenize_parens,
    text: "((())) )abc(+",
    tokens: (
      ParenL,
      ParenL,
      ParenL,
      ParenR,
      ParenR,
      ParenR,
      Whitespace,
      ParenR,
      Identifier:"abc",
      ParenL,
      Plus,
    ),
  }

  test! {
    name: crlf_newline,
    text: "#\r\n#asdf\r\n",
    tokens: (
      Comment:"#",
      Eol:"\r\n",
      Comment:"#asdf",
      Eol:"\r\n",
    ),
  }

  test! {
    name: multiple_recipes,
    text: "a:\n  foo\nb:",
    tokens: (
      Identifier:"a",
      Colon,
      Eol,
      Indent:"  ",
      Text:"foo",
      Eol,
      Dedent,
      Identifier:"b",
      Colon,
    ),
  }

  test! {
    name:   brackets,
    text:   "][",
    tokens: (BracketR, BracketL),
  }

  error! {
    name:  tokenize_space_then_tab,
    input: "a:
 0
 1
\t2
",
    offset: 9,
    line:   3,
    column: 0,
    width:  1,
    kind:   InconsistentLeadingWhitespace{expected: " ", found: "\t"},
  }

  error! {
    name:  tokenize_tabs_then_tab_space,
    input: "a:
\t\t0
\t\t 1
\t  2
",
    offset: 12,
    line:   3,
    column: 0,
    width:  2,
    kind:   InconsistentLeadingWhitespace{expected: "\t\t", found: "\t "},
  }

  error! {
    name:   tokenize_unknown,
    input:  "~",
    offset: 0,
    line:   0,
    column: 0,
    width:  1,
    kind:   UnknownStartOfToken,
  }

  error! {
    name:   unterminated_string_with_escapes,
    input:  r#"a = "\n\t\r\"\\"#,
    offset: 4,
    line:   0,
    column: 4,
    width:  1,
    kind:   UnterminatedString,
  }

  error! {
    name:   unterminated_raw_string,
    input:  "r a='asdf",
    offset: 4,
    line:   0,
    column: 4,
    width:  1,
    kind:   UnterminatedString,
  }

  error! {
    name:   unterminated_interpolation,
    input:  "foo:\n echo {{
  ",
    offset: 11,
    line:   1,
    column: 6,
    width:  2,
    kind:   UnterminatedInterpolation,
  }

  error! {
    name:   unterminated_backtick,
    input:  "`echo",
    offset: 0,
    line:   0,
    column: 0,
    width:  1,
    kind:   UnterminatedBacktick,
  }

  error! {
    name:   unpaired_carriage_return,
    input:  "foo\rbar",
    offset: 3,
    line:   0,
    column: 3,
    width:  1,
    kind:   UnpairedCarriageReturn,
  }

  error! {
    name:   unknown_start_of_token_ampersand,
    input:  " \r\n&",
    offset: 3,
    line:   1,
    column: 0,
    width:  1,
    kind:   UnknownStartOfToken,
  }

  error! {
    name:   unknown_start_of_token_tilde,
    input:  "~",
    offset: 0,
    line:   0,
    column: 0,
    width:  1,
    kind:   UnknownStartOfToken,
  }

  error! {
    name:   invalid_name_start_dash,
    input:  "-foo",
    offset: 0,
    line:   0,
    column: 0,
    width:  1,
    kind:   UnknownStartOfToken,
  }

  error! {
    name:   invalid_name_start_digit,
    input:  "0foo",
    offset: 0,
    line:   0,
    column: 0,
    width:  1,
    kind:   UnknownStartOfToken,
  }

  error! {
    name:   unterminated_string,
    input:  r#"a = ""#,
    offset: 4,
    line:   0,
    column: 4,
    width:  1,
    kind:   UnterminatedString,
  }

  error! {
    name:   mixed_leading_whitespace,
    input:  "a:\n\t echo hello",
    offset: 3,
    line:   1,
    column: 0,
    width:  2,
    kind:   MixedLeadingWhitespace{whitespace: "\t "},
  }

  error! {
    name:   unclosed_interpolation_delimiter,
    input:  "a:\n echo {{ foo",
    offset: 9,
    line:   1,
    column: 6,
    width:  2,
    kind:   UnterminatedInterpolation,
  }
}
