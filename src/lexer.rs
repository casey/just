use crate::common::*;

use CompilationErrorKind::*;
use TokenKind::*;

/// Just language lexer
///
/// `self.next` points to the next character to be lexed, and
/// the text between `self.token_start` and `self.token_end` contains
/// the current token being lexed.
pub(crate) struct Lexer<'a> {
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

impl<'a> Lexer<'a> {
  /// Lex `text`
  pub(crate) fn lex(text: &str) -> CompilationResult<Vec<Token>> {
    Lexer::new(text).tokenize()
  }

  /// Create a new Lexer to lex `text`
  fn new(text: &'a str) -> Lexer<'a> {
    let mut chars = text.chars();
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
      text,
    }
  }

  /// Advance over the chracter in `self.next`, updating
  /// `self.token_end` accordingly.
  fn advance(&mut self) -> CompilationResult<'a, ()> {
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
  fn lexeme(&self) -> &'a str {
    &self.text[self.token_start.offset..self.token_end.offset]
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
  fn rest(&self) -> &'a str {
    &self.text[self.token_end.offset..]
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
  fn token(&mut self, kind: TokenKind) {
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
      width: 0,
      kind: CompilationErrorKind::Internal {
        message: message.into(),
      },
    }
  }

  /// Create a compilation error with `kind`
  fn error(&self, kind: CompilationErrorKind<'a>) -> CompilationError<'a> {
    // Use the in-progress token span as the location of the error.

    // The width of the error site to highlight depends on the kind of error:
    let width = match kind {
      // highlight ' or "
      UnterminatedString => 1,
      // highlight `
      UnterminatedBacktick => 1,
      // highlight the full token
      _ => self.lexeme().len(),
    };

    CompilationError {
      text: self.text,
      offset: self.token_start.offset,
      line: self.token_start.line,
      column: self.token_start.column,
      width,
      kind,
    }
  }

  fn unterminated_interpolation_error(
    &self,
    interpolation_start: Position,
  ) -> CompilationError<'a> {
    CompilationError {
      text: self.text,
      offset: interpolation_start.offset,
      line: interpolation_start.line,
      column: interpolation_start.column,
      width: 2,
      kind: UnterminatedInterpolation,
    }
  }

  /// Consume the text and produce a series of tokens
  fn tokenize(mut self) -> CompilationResult<'a, Vec<Token<'a>>> {
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

    Ok(self.tokens)
  }

  /// Handle blank lines and indentation
  fn lex_line_start(&mut self) -> CompilationResult<'a, ()> {
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
  fn lex_normal(&mut self, start: char) -> CompilationResult<'a, ()> {
    match start {
      '@' => self.lex_single(At),
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
      'a'..='z' | 'A'..='Z' | '_' => self.lex_name(),
      _ => {
        self.advance()?;
        Err(self.error(UnknownStartOfToken))
      }
    }
  }

  /// Lex token beginning with `start` in interpolation state
  fn lex_interpolation(
    &mut self,
    interpolation_start: Position,
    start: char,
  ) -> CompilationResult<'a, ()> {
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
  fn lex_text(&mut self) -> CompilationResult<'a, ()> {
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
        self.state.push(State::Interpolation {
          interpolation_start: self.token_start,
        });
        self.lex_double(InterpolationStart)
      }
      EndOfFile => self.pop_state(),
    }
  }

  /// Lex token beginning with `start` in indented state
  fn lex_indented(&mut self) -> CompilationResult<'a, ()> {
    self.state.push(State::Text);
    self.token(Line);
    Ok(())
  }

  /// Lex a single character token
  fn lex_single(&mut self, kind: TokenKind) -> CompilationResult<'a, ()> {
    self.advance()?;
    self.token(kind);
    Ok(())
  }

  /// Lex a double character token
  fn lex_double(&mut self, kind: TokenKind) -> CompilationResult<'a, ()> {
    self.advance()?;
    self.advance()?;
    self.token(kind);
    Ok(())
  }

  /// Lex a token starting with ':'
  fn lex_colon(&mut self) -> CompilationResult<'a, ()> {
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
  fn lex_brace_l(&mut self) -> CompilationResult<'a, ()> {
    if !self.rest_starts_with("{{") {
      self.advance()?;

      return Err(self.error(UnknownStartOfToken));
    }

    self.lex_double(InterpolationStart)
  }

  /// Lex a token starting with '}'
  fn lex_brace_r(&mut self) -> CompilationResult<'a, ()> {
    if !self.rest_starts_with("}}") {
      self.advance()?;

      return Err(self.error(UnknownStartOfToken));
    }

    self.lex_double(InterpolationEnd)
  }

  /// Lex a carriage return and line feed
  fn lex_cr_lf(&mut self) -> CompilationResult<'a, ()> {
    if !self.rest_starts_with("\r\n") {
      // advance over \r
      self.advance()?;

      return Err(self.error(UnpairedCarriageReturn));
    }

    self.lex_double(Eol)
  }

  /// Lex name: [a-zA-Z_][a-zA-Z0-9_]*
  fn lex_name(&mut self) -> CompilationResult<'a, ()> {
    while self
      .next
      .map(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
      .unwrap_or(false)
    {
      self.advance()?;
    }

    self.token(Name);

    Ok(())
  }

  /// Lex comment: #[^\r\n]
  fn lex_comment(&mut self) -> CompilationResult<'a, ()> {
    // advance over #
    self.advance()?;

    while !self.at_eol_or_eof() {
      self.advance()?;
    }

    self.token(Comment);

    Ok(())
  }

  /// Lex backtick: `[^\r\n]*`
  fn lex_backtick(&mut self) -> CompilationResult<'a, ()> {
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
  fn lex_whitespace(&mut self) -> CompilationResult<'a, ()> {
    while self.next_is_whitespace() {
      self.advance()?
    }

    self.token(Whitespace);

    Ok(())
  }

  /// Lex raw string: '[^']*'
  fn lex_raw_string(&mut self) -> CompilationResult<'a, ()> {
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
  fn lex_cooked_string(&mut self) -> CompilationResult<'a, ()> {
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

  fn summary(tokens: &[Token]) -> String {
    use TokenKind::*;

    tokens
      .iter()
      .map(|t| match t.kind {
        At => "@",
        Backtick => "`",
        Colon => ":",
        ColonEquals => ":=",
        Comma => ",",
        Comment => "#",
        Dedent => "<",
        Eof => ".",
        Eol => "$",
        Equals => "=",
        Indent => ">",
        InterpolationEnd => "}",
        InterpolationStart => "{",
        Line => "^",
        Name => "N",
        ParenL => "(",
        ParenR => ")",
        Plus => "+",
        StringRaw => "'",
        StringCooked => "\"",
        Text => "_",
        Whitespace => " ",
      })
      .collect::<Vec<&str>>()
      .join("")
  }

  macro_rules! lex_test {
    ($name:ident, $input:expr, $expected:expr $(,)*) => {
      #[test]
      fn $name() {
        let input = $input;
        let expected = $expected;
        let tokens = Lexer::lex(input).unwrap();
        let roundtrip = tokens
          .iter()
          .map(Token::lexeme)
          .collect::<Vec<&str>>()
          .join("");
        let actual = summary(&tokens);

        use pretty_assertions::assert_eq;

        assert_eq!(actual, expected, "token summary mismatch");

        assert_eq!(input, roundtrip, "token round-trip not equal");
      }
    };
  }

  lex_test! {
    name,
    "foo",
    "N.",
  }

  lex_test! {
    comment,
    "# hello",
    "#.",
  }

  lex_test! {
    backtick,
    "`echo`",
    "`.",
  }

  lex_test! {
    raw_string,
    "'hello'",
    "'.",
  }

  lex_test! {
    cooked_string,
    r#""hello""#,
    r#""."#,
  }

  lex_test! {
    export_concatination,
    "export foo = 'foo' + 'bar'",
    "N N = ' + '.",
  }

  lex_test! {
    export_complex,
    "export foo = ('foo' + 'bar') + `baz`",
    "N N = (' + ') + `.",
  }

  lex_test! {
    eol_linefeed,
    "\n",
    "$.",
  }

  lex_test! {
    eol_carriage_return_linefeed,
    "\r\n",
    "$.",
  }

  lex_test! {
    indented_line,
    "foo:\n a",
    "N:$>^_<.",
  }

  lex_test! {
    indented_block,
    r##"foo:
  a
  b
  c
"##,
    "N:$>^_$ ^_$ ^_$<.",
  }

  lex_test! {
    indented_block_followed_by_item,
    "foo:
  a
b:",
    "N:$>^_$<N:.",
  }

  lex_test! {
    indented_block_followed_by_blank,
    "foo:
    a

b:",
      "N:$>^_$^$<N:.",
  }

  lex_test! {
    indented_line_containing_unpaired_carriage_return,
    "foo:\n \r \n",
    "N:$>^_$<.",
  }

  lex_test! {
    indented_blocks,
    "
b: a
  @mv a b

a:
  @touch F
  @touch a

d: c
  @rm c

c: b
  @mv b c",
    "$N: N$>^_$^$<N:$>^_$ ^_$^$<N: N$>^_$^$<N: N$>^_<.",
  }

  lex_test! {
    interpolation_empty,
    "hello:\n echo {{}}",
    "N:$>^_{}<.",
  }

  lex_test! {
    interpolation_expression,
    "hello:\n echo {{`echo hello` + `echo goodbye`}}",
    "N:$>^_{` + `}<.",
  }

  lex_test! {
    tokenize_names,
    "\
foo
bar-bob
b-bob_asdfAAAA
test123",
    "N$N$N$N.",
  }

  lex_test! {
    tokenize_indented_line,
    "foo:\n a",
    "N:$>^_<.",
  }

  lex_test! {
    tokenize_indented_block,
    r##"foo:
  a
  b
  c
"##,
    "N:$>^_$ ^_$ ^_$<.",
  }

  lex_test! {
    tokenize_strings,
    r#"a = "'a'" + '"b"' + "'c'" + '"d"'#echo hello"#,
    r#"N = " + ' + " + '#."#,
  }

  lex_test! {
    tokenize_recipe_interpolation_eol,
    "foo: # some comment
 {{hello}}
",
    "N: #$>^{N}$<.",
  }

  lex_test! {
    tokenize_recipe_interpolation_eof,
    "foo: # more comments
 {{hello}}
# another comment
",
    "N: #$>^{N}$<#$.",
  }

  lex_test! {
    tokenize_recipe_complex_interpolation_expression,
    "foo: #lol\n {{a + b + \"z\" + blarg}}",
    "N: #$>^{N + N + \" + N}<.",
  }

  lex_test! {
    tokenize_recipe_multiple_interpolations,
    "foo:,#ok\n {{a}}0{{b}}1{{c}}",
    "N:,#$>^{N}_{N}_{N}<.",
  }

  lex_test! {
    tokenize_junk,
    "bob

hello blah blah blah : a b c #whatever
    ",
    "N$$N N N N : N N N #$ .",
  }

  lex_test! {
    tokenize_empty_lines,
    "
# this does something
hello:
  asdf
  bsdf

  csdf

  dsdf # whatever

# yolo
  ",
    "$#$N:$>^_$ ^_$^$ ^_$^$ ^_$^$<#$ .",
  }

  lex_test! {
    tokenize_comment_before_variable,
    "
#
A='1'
echo:
  echo {{A}}
  ",
    "$#$N='$N:$>^_{N}$ <.",
  }

  lex_test! {
    tokenize_interpolation_backticks,
    "hello:\n echo {{`echo hello` + `echo goodbye`}}",
    "N:$>^_{` + `}<.",
  }

  lex_test! {
    tokenize_empty_interpolation,
    "hello:\n echo {{}}",
    "N:$>^_{}<.",
  }

  lex_test! {
    tokenize_assignment_backticks,
    "a = `echo hello` + `echo goodbye`",
    "N = ` + `.",
  }

  lex_test! {
    tokenize_multiple,
    "
hello:
  a
  b

  c

  d

# hello
bob:
  frank
 \t",

    "$N:$>^_$ ^_$^$ ^_$^$ ^_$^$<#$N:$>^_$ <.",
  }

  lex_test! {
    tokenize_comment,
    "a:=#",
    "N:=#."
  }

  lex_test! {
    tokenize_comment_with_bang,
    "a:=#foo!",
    "N:=#."
  }

  lex_test! {
    tokenize_order,
    r"
b: a
  @mv a b

a:
  @touch F
  @touch a

d: c
  @rm c

c: b
  @mv b c",
    "$N: N$>^_$^$<N:$>^_$ ^_$^$<N: N$>^_$^$<N: N$>^_<.",
  }

  lex_test! {
    tokenize_parens,
    r"((())) )abc(+",
    "((())) )N(+.",
  }

  lex_test! {
    crlf_newline,
    "#\r\n#asdf\r\n",
    "#$#$.",
  }

  lex_test! {
    multiple_recipes,
    "a:\n  foo\nb:",
    "N:$>^_$<N:.",
  }

  error_test! {
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

  error_test! {
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

  error_test! {
    name:   tokenize_unknown,
    input:  "~",
    offset: 0,
    line:   0,
    column: 0,
    width:  1,
    kind:   UnknownStartOfToken,
  }

  error_test! {
    name:   unterminated_string_with_escapes,
    input:  r#"a = "\n\t\r\"\\"#,
    offset: 4,
    line:   0,
    column: 4,
    width:  1,
    kind:   UnterminatedString,
  }

  error_test! {
    name:   unterminated_raw_string,
    input:  "r a='asdf",
    offset: 4,
    line:   0,
    column: 4,
    width:  1,
    kind:   UnterminatedString,
  }

  error_test! {
    name:   unterminated_interpolation,
    input:  "foo:\n echo {{
  ",
    offset: 11,
    line:   1,
    column: 6,
    width:  2,
    kind:   UnterminatedInterpolation,
  }

  error_test! {
    name:   unterminated_backtick,
    input:  "`echo",
    offset: 0,
    line:   0,
    column: 0,
    width:  1,
    kind:   UnterminatedBacktick,
  }

  error_test! {
    name:   unpaired_carriage_return,
    input:  "foo\rbar",
    offset: 3,
    line:   0,
    column: 3,
    width:  1,
    kind:   UnpairedCarriageReturn,
  }

  error_test! {
    name:   unknown_start_of_token_ampersand,
    input:  " \r\n&",
    offset: 3,
    line:   1,
    column: 0,
    width:  1,
    kind:   UnknownStartOfToken,
  }

  error_test! {
    name:   unknown_start_of_token_tilde,
    input:  "~",
    offset: 0,
    line:   0,
    column: 0,
    width:  1,
    kind:   UnknownStartOfToken,
  }

  error_test! {
    name:   unterminated_string,
    input:  r#"a = ""#,
    offset: 4,
    line:   0,
    column: 4,
    width:  1,
    kind:   UnterminatedString,
  }

  error_test! {
    name:   mixed_leading_whitespace,
    input:  "a:\n\t echo hello",
    offset: 3,
    line:   1,
    column: 0,
    width:  2,
    kind:   MixedLeadingWhitespace{whitespace: "\t "},
  }

  error_test! {
    name:   unclosed_interpolation_delimiter,
    input:  "a:\n echo {{ foo",
    offset: 9,
    line:   1,
    column: 6,
    width:  2,
    kind:   UnterminatedInterpolation,
  }
}
