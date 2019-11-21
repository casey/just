use crate::common::*;

use TokenKind::*;

/// Just language parser
///
/// The parser is a (hopefully) straightforward recursive descent parser.
///
/// It uses a few tokens of lookahead to disambiguate different constructs.
///
/// The `expect_*` and `presume_`* methods are similar in that they assert
/// the type of unparsed tokens and consume them. However, upon encountering
/// an unexpected token, the `expect_*` methods return an unexpected token
/// error, whereas the `presume_*` tokens return an internal error.
///
/// The `presume_*` methods are used when the token stream has been inspected
/// in some other way, and thus encountering an unexpected token is a bug in
/// Just, and not a syntax error.
///
/// All methods starting with `parse_*` parse and return a language construct.
pub(crate) struct Parser<'tokens, 'src> {
  /// Source tokens
  tokens: &'tokens [Token<'src>],
  /// Index of the next un-parsed token
  next: usize,
}

impl<'tokens, 'src> Parser<'tokens, 'src> {
  /// Parse `tokens` into an `Module`
  pub(crate) fn parse(tokens: &'tokens [Token<'src>]) -> CompilationResult<'src, Module<'src>> {
    Self::new(tokens).parse_justfile()
  }

  /// Construct a new Paser from a token stream
  fn new(tokens: &'tokens [Token<'src>]) -> Parser<'tokens, 'src> {
    Parser { next: 0, tokens }
  }

  fn error(
    &self,
    kind: CompilationErrorKind<'src>,
  ) -> CompilationResult<'src, CompilationError<'src>> {
    Ok(self.next()?.error(kind))
  }

  /// Construct an unexpected token error with the token returned by `Parser::next`
  fn unexpected_token(
    &self,
    expected: &[TokenKind],
  ) -> CompilationResult<'src, CompilationError<'src>> {
    let mut expected = expected.to_vec();
    expected.sort();

    self.error(CompilationErrorKind::UnexpectedToken {
      expected,
      found: self.next()?.kind,
    })
  }

  fn internal_error(
    &self,
    message: impl Into<String>,
  ) -> CompilationResult<'src, CompilationError<'src>> {
    self.error(CompilationErrorKind::Internal {
      message: message.into(),
    })
  }

  /// An iterator over the remaining significant tokens
  fn rest(&self) -> impl Iterator<Item = Token<'src>> + 'tokens {
    self.tokens[self.next..]
      .iter()
      .cloned()
      .filter(|token| token.kind != Whitespace)
  }

  /// The next significant token
  fn next(&self) -> CompilationResult<'src, Token<'src>> {
    if let Some(token) = self.rest().next() {
      Ok(token)
    } else {
      Err(self.internal_error("`Parser::next()` called after end of token stream")?)
    }
  }

  /// Check if the next significant token is of kind `kind`
  fn next_is(&self, kind: TokenKind) -> bool {
    self.next_are(&[kind])
  }

  /// Check if the next significant tokens are of kinds `kinds`
  fn next_are(&self, kinds: &[TokenKind]) -> bool {
    let mut rest = self.rest();
    for kind in kinds {
      match rest.next() {
        Some(token) => {
          if token.kind != *kind {
            return false;
          }
        }
        None => return false,
      }
    }
    true
  }

  /// Get the `n`th next significant token
  fn get(&self, n: usize) -> CompilationResult<'src, Token<'src>> {
    match self.rest().nth(n) {
      Some(token) => Ok(token),
      None => Err(self.internal_error("`Parser::get()` advanced past end of token stream")?),
    }
  }

  /// Advance past one significant token
  fn advance(&mut self) -> CompilationResult<'src, Token<'src>> {
    for skipped in &self.tokens[self.next..] {
      self.next += 1;

      if skipped.kind != Whitespace {
        return Ok(*skipped);
      }
    }

    Err(self.internal_error("`Parser::advance()` advanced past end of token stream")?)
  }

  /// Return the next token if it is of kind `expected`, otherwise, return an
  /// unexpected token error
  fn expect(&mut self, expected: TokenKind) -> CompilationResult<'src, Token<'src>> {
    if let Some(token) = self.accept(expected)? {
      Ok(token)
    } else {
      Err(self.unexpected_token(&[expected])?)
    }
  }

  /// Return an error if the next token is not one of kinds `kinds`.
  fn expect_any(&mut self, expected: &[TokenKind]) -> CompilationResult<'src, Token<'src>> {
    for expected in expected.iter().cloned() {
      if let Some(token) = self.accept(expected)? {
        return Ok(token);
      }
    }

    Err(self.unexpected_token(expected)?)
  }

  /// Return an unexpected token error if the next token is not an EOL
  fn expect_eol(&mut self) -> CompilationResult<'src, ()> {
    self.accept(Comment)?;

    if self.next_is(Eof) {
      return Ok(());
    }

    self.expect(Eol).map(|_| ()).expected(&[Eof])
  }

  /// Return an internal error if the next token is not of kind `Identifier` with
  /// lexeme `lexeme`.
  fn presume_name(&mut self, lexeme: &str) -> CompilationResult<'src, ()> {
    let next = self.advance()?;

    if next.kind != Identifier {
      Err(self.internal_error(format!(
        "Presumed next token would have kind {}, but found {}",
        Identifier, next.kind
      ))?)
    } else if next.lexeme() != lexeme {
      Err(self.internal_error(format!(
        "Presumed next token would have lexeme \"{}\", but found \"{}\"",
        lexeme,
        next.lexeme(),
      ))?)
    } else {
      Ok(())
    }
  }

  /// Return an internal error if the next token is not of kind `kind`.
  fn presume(&mut self, kind: TokenKind) -> CompilationResult<'src, Token<'src>> {
    let next = self.advance()?;

    if next.kind != kind {
      Err(self.internal_error(format!(
        "Presumed next token would have kind {:?}, but found {:?}",
        kind, next.kind
      ))?)
    } else {
      Ok(next)
    }
  }

  /// Return an internal error if the next token is not one of kinds `kinds`.
  fn presume_any(&mut self, kinds: &[TokenKind]) -> CompilationResult<'src, Token<'src>> {
    let next = self.advance()?;
    if !kinds.contains(&next.kind) {
      Err(self.internal_error(format!(
        "Presumed next token would be {}, but found {}",
        List::or(kinds),
        next.kind
      ))?)
    } else {
      Ok(next)
    }
  }

  /// Accept and return a token of kind `kind`
  fn accept(&mut self, kind: TokenKind) -> CompilationResult<'src, Option<Token<'src>>> {
    let next = self.next()?;
    if next.kind == kind {
      self.advance()?;
      Ok(Some(next))
    } else {
      Ok(None)
    }
  }

  /// Accept a token of kind `Identifier` and parse into an `Name`
  fn accept_name(&mut self) -> CompilationResult<'src, Option<Name<'src>>> {
    if self.next_is(Identifier) {
      Ok(Some(self.parse_name()?))
    } else {
      Ok(None)
    }
  }

  /// Accept and return `true` if next token is of kind `kind`
  fn accepted(&mut self, kind: TokenKind) -> CompilationResult<'src, bool> {
    Ok(self.accept(kind)?.is_some())
  }

  /// Parse a justfile, consumes self
  fn parse_justfile(mut self) -> CompilationResult<'src, Module<'src>> {
    let mut items = Vec::new();
    let mut warnings = Vec::new();

    let mut doc = None;

    loop {
      let next = self.next()?;

      match next.kind {
        Comment => {
          doc = Some(next.lexeme()[1..].trim());
          self.expect_eol()?;
        }
        Eol => {
          self.advance()?;
        }
        Eof => {
          self.advance()?;
          break;
        }
        Identifier => match next.lexeme() {
          keyword::ALIAS => {
            if self.next_are(&[Identifier, Identifier, Equals]) {
              warnings.push(Warning::DeprecatedEquals {
                equals: self.get(2)?,
              });
              items.push(Item::Alias(self.parse_alias()?));
            } else if self.next_are(&[Identifier, Identifier, ColonEquals]) {
              items.push(Item::Alias(self.parse_alias()?));
            } else {
              items.push(Item::Recipe(self.parse_recipe(doc, false)?));
            }
          }
          keyword::EXPORT => {
            if self.next_are(&[Identifier, Identifier, Equals]) {
              warnings.push(Warning::DeprecatedEquals {
                equals: self.get(2)?,
              });
              self.presume_name(keyword::EXPORT)?;
              items.push(Item::Assignment(self.parse_assignment(true)?));
            } else if self.next_are(&[Identifier, Identifier, ColonEquals]) {
              self.presume_name(keyword::EXPORT)?;
              items.push(Item::Assignment(self.parse_assignment(true)?));
            } else {
              items.push(Item::Recipe(self.parse_recipe(doc, false)?));
            }
          }
          keyword::SET => {
            if self.next_are(&[Identifier, Identifier, ColonEquals]) {
              items.push(Item::Set(self.parse_set()?));
            } else {
              items.push(Item::Recipe(self.parse_recipe(doc, false)?));
            }
          }
          _ => {
            if self.next_are(&[Identifier, Equals]) {
              warnings.push(Warning::DeprecatedEquals {
                equals: self.get(1)?,
              });
              items.push(Item::Assignment(self.parse_assignment(false)?));
            } else if self.next_are(&[Identifier, ColonEquals]) {
              items.push(Item::Assignment(self.parse_assignment(false)?));
            } else {
              items.push(Item::Recipe(self.parse_recipe(doc, false)?));
            }
          }
        },
        At => {
          self.presume(At)?;
          items.push(Item::Recipe(self.parse_recipe(doc, true)?));
        }
        _ => {
          return Err(self.unexpected_token(&[Identifier, At])?);
        }
      }

      if next.kind != Comment {
        doc = None;
      }
    }

    if self.next != self.tokens.len() {
      Err(self.internal_error(format!(
        "Parse completed with {} unparsed tokens",
        self.tokens.len() - self.next,
      ))?)
    } else {
      Ok(Module { items, warnings })
    }
  }

  /// Parse an alias, e.g `alias name := target`
  fn parse_alias(&mut self) -> CompilationResult<'src, Alias<'src, Name<'src>>> {
    self.presume_name(keyword::ALIAS)?;
    let name = self.parse_name()?;
    self.presume_any(&[Equals, ColonEquals])?;
    let target = self.parse_name()?;
    self.expect_eol()?;
    Ok(Alias { name, target })
  }

  /// Parse an assignment, e.g. `foo := bar`
  fn parse_assignment(&mut self, export: bool) -> CompilationResult<'src, Assignment<'src>> {
    let name = self.parse_name()?;
    self.presume_any(&[Equals, ColonEquals])?;
    let expression = self.parse_expression()?;
    self.expect_eol()?;
    Ok(Assignment {
      name,
      export,
      expression,
    })
  }

  /// Parse an expression, e.g. `1 + 2`
  fn parse_expression(&mut self) -> CompilationResult<'src, Expression<'src>> {
    let value = self.parse_value()?;

    if self.accepted(Plus)? {
      let lhs = Box::new(value);
      let rhs = Box::new(self.parse_expression()?);
      Ok(Expression::Concatination { lhs, rhs })
    } else {
      Ok(value)
    }
  }

  /// Parse a value, e.g. `(bar)`
  fn parse_value(&mut self) -> CompilationResult<'src, Expression<'src>> {
    let next = self.next()?;

    match next.kind {
      StringCooked | StringRaw => Ok(Expression::StringLiteral {
        string_literal: self.parse_string_literal()?,
      }),
      Backtick => {
        let contents = &next.lexeme()[1..next.lexeme().len() - 1];
        let token = self.advance()?;
        Ok(Expression::Backtick { contents, token })
      }
      Identifier => {
        let name = self.parse_name()?;

        if self.next_is(ParenL) {
          let arguments = self.parse_sequence()?;
          Ok(Expression::Call {
            function: name,
            arguments,
          })
        } else {
          Ok(Expression::Variable { name })
        }
      }
      ParenL => {
        self.presume(ParenL)?;
        let contents = Box::new(self.parse_expression()?);
        self.expect(ParenR)?;
        Ok(Expression::Group { contents })
      }
      _ => Err(self.unexpected_token(&[StringCooked, StringRaw, Backtick, Identifier, ParenL])?),
    }
  }

  /// Parse a string literal, e.g. `"FOO"`
  fn parse_string_literal(&mut self) -> CompilationResult<'src, StringLiteral<'src>> {
    let token = self.expect_any(&[StringRaw, StringCooked])?;

    let raw = &token.lexeme()[1..token.lexeme().len() - 1];

    match token.kind {
      StringRaw => Ok(StringLiteral {
        raw,
        cooked: Cow::Borrowed(raw),
      }),
      StringCooked => {
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
          } else if c == '\\' {
            escape = true;
          } else {
            cooked.push(c);
          }
        }
        Ok(StringLiteral {
          raw,
          cooked: Cow::Owned(cooked),
        })
      }
      _ => Err(token.error(CompilationErrorKind::Internal {
        message: "`Parser::parse_string_literal` called on non-string token".to_string(),
      })),
    }
  }

  /// Parse a name from an identifier token
  fn parse_name(&mut self) -> CompilationResult<'src, Name<'src>> {
    self.expect(Identifier).map(Name::from_identifier)
  }

  /// Parse sequence of comma-separated expressions
  fn parse_sequence(&mut self) -> CompilationResult<'src, Vec<Expression<'src>>> {
    self.presume(ParenL)?;

    let mut elements = Vec::new();

    while !self.next_is(ParenR) {
      elements.push(self.parse_expression().expected(&[ParenR])?);

      if !self.accepted(Comma)? {
        break;
      }
    }

    self.expect(ParenR)?;

    Ok(elements)
  }

  /// Parse a recipe
  fn parse_recipe(
    &mut self,
    doc: Option<&'src str>,
    quiet: bool,
  ) -> CompilationResult<'src, Recipe<'src, Name<'src>>> {
    let name = self.parse_name()?;

    let mut positional = Vec::new();

    while self.next_is(Identifier) {
      positional.push(self.parse_parameter(false)?);
    }

    let variadic = if self.accepted(Plus)? {
      let variadic = self.parse_parameter(true)?;

      if let Some(identifier) = self.accept(Identifier)? {
        return Err(
          identifier.error(CompilationErrorKind::ParameterFollowsVariadicParameter {
            parameter: identifier.lexeme(),
          }),
        );
      }

      Some(variadic)
    } else {
      None
    };

    let result = self.expect(Colon);

    if result.is_err() {
      let mut alternatives = Vec::new();

      if variadic.is_none() {
        alternatives.push(Identifier);
      }

      if !quiet && variadic.is_none() && positional.is_empty() {
        alternatives.push(ColonEquals);
      }

      if variadic.is_some() || !positional.is_empty() {
        alternatives.push(Equals);
      }

      if variadic.is_none() {
        alternatives.push(Plus);
      }

      result.expected(&alternatives)?;
    }

    let mut dependencies = Vec::new();

    while let Some(dependency) = self.accept_name()? {
      dependencies.push(dependency);
    }

    self.expect_eol().expected(&[Identifier])?;

    let body = self.parse_body()?;

    Ok(Recipe {
      private: name.lexeme().starts_with('_'),
      shebang: body.first().map(Line::is_shebang).unwrap_or(false),
      parameters: positional.into_iter().chain(variadic).collect(),
      doc,
      name,
      quiet,
      dependencies,
      body,
    })
  }

  /// Parse a recipe parameter
  fn parse_parameter(&mut self, variadic: bool) -> CompilationResult<'src, Parameter<'src>> {
    let name = self.parse_name()?;

    let default = if self.accepted(Equals)? {
      Some(self.parse_value()?)
    } else {
      None
    };

    Ok(Parameter {
      name,
      default,
      variadic,
    })
  }

  /// Parse the body of a recipe
  fn parse_body(&mut self) -> CompilationResult<'src, Vec<Line<'src>>> {
    let mut lines = Vec::new();

    if self.accepted(Indent)? {
      while !self.accepted(Dedent)? {
        let line = if self.accepted(Eol)? {
          Line {
            fragments: Vec::new(),
          }
        } else {
          let mut fragments = Vec::new();

          while !(self.accepted(Eol)? || self.next_is(Dedent)) {
            if let Some(token) = self.accept(Text)? {
              fragments.push(Fragment::Text { token });
            } else if self.accepted(InterpolationStart)? {
              fragments.push(Fragment::Interpolation {
                expression: self.parse_expression()?,
              });
              self.expect(InterpolationEnd)?;
            } else {
              return Err(self.unexpected_token(&[Text, InterpolationStart])?);
            }
          }

          Line { fragments }
        };

        lines.push(line);
      }
    }

    while lines.last().map(Line::is_empty).unwrap_or(false) {
      lines.pop();
    }

    Ok(lines)
  }

  /// Parse a setting
  fn parse_set(&mut self) -> CompilationResult<'src, Set<'src>> {
    self.presume_name(keyword::SET)?;
    let name = Name::from_identifier(self.presume(Identifier)?);
    self.presume(ColonEquals)?;
    match name.lexeme() {
      keyword::SHELL => {
        self.expect(BracketL)?;

        let command = self.parse_string_literal()?;

        let mut arguments = Vec::new();

        let mut comma = false;

        if self.accepted(Comma)? {
          comma = true;
          while !self.next_is(BracketR) {
            arguments.push(self.parse_string_literal().expected(&[BracketR])?);

            if !self.accepted(Comma)? {
              comma = false;
              break;
            }
            comma = true;
          }
        }

        self
          .expect(BracketR)
          .expected(if comma { &[] } else { &[Comma] })?;

        Ok(Set {
          value: Setting::Shell(setting::Shell { command, arguments }),
          name,
        })
      }
      _ => Err(name.error(CompilationErrorKind::UnknownSetting {
        setting: name.lexeme(),
      })),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use pretty_assertions::assert_eq;
  use testing::unindent;
  use CompilationErrorKind::*;

  macro_rules! test {
    {
      name: $name:ident,
      text: $text:expr,
      tree: $tree:tt,
    } => {
      #[test]
      fn $name() {
        let text: String = $text.into();
        let want = tree!($tree);
        test(&text, want);
      }
    }
  }

  fn test(text: &str, want: Tree) {
    let unindented = unindent(text);
    let tokens = Lexer::lex(&unindented).expect("lexing failed");
    let justfile = Parser::parse(&tokens).expect("parsing failed");
    let have = justfile.tree();
    if have != want {
      println!("parsed text: {}", unindented);
      println!("expected:    {}", want);
      println!("but got:     {}", have);
      println!("tokens:      {:?}", tokens);
      panic!();
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
    let tokens = Lexer::lex(src).expect("Lexing failed in parse test...");

    match Parser::parse(&tokens) {
      Ok(_) => panic!("Parsing unexpectedly succeeded"),
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
    name: empty,
    text: "",
    tree: (justfile),
  }

  test! {
    name: empty_multiline,
    text: "





    ",
    tree: (justfile),
  }

  test! {
    name: whitespace,
    text: " ",
    tree: (justfile),
  }

  test! {
    name: alias_single,
    text: "alias t := test",
    tree: (justfile (alias t test)),
  }

  test! {
    name: aliases_multiple,
    text: "alias t := test\nalias b := build",
    tree: (
      justfile
      (alias t test)
      (alias b build)
    ),
  }

  test! {
    name: alias_equals,
    text: "alias t = test",
    tree: (justfile
      (alias t test)
      (warning deprecated_equals)
    ),
  }

  test! {
    name: export,
    text: r#"export x := "hello""#,
    tree: (justfile (assignment #export x "hello")),
  }

  test! {
    name: export_equals,
    text: r#"export x = "hello""#,
    tree: (justfile
      (assignment #export x "hello")
      (warning deprecated_equals)
    ),
  }

  test! {
    name: assignment,
    text: r#"x := "hello""#,
    tree: (justfile (assignment x "hello")),
  }

  test! {
    name: assignment_equals,
    text: r#"x = "hello""#,
    tree: (justfile
      (assignment x "hello")
      (warning deprecated_equals)
    ),
  }

  test! {
    name: backtick,
    text: "x := `hello`",
    tree: (justfile (assignment x (backtick "hello"))),
  }

  test! {
    name: variable,
    text: "x := y",
    tree: (justfile (assignment x y)),
  }

  test! {
    name: group,
    text: "x := (y)",
    tree: (justfile (assignment x (y))),
  }

  test! {
    name: addition_single,
    text: "x := a + b",
    tree: (justfile (assignment x (+ a b))),
  }

  test! {
    name: addition_chained,
    text: "x := a + b + c",
    tree: (justfile (assignment x (+ a (+ b c)))),
  }

  test! {
    name: call_one_arg,
    text: "x := foo(y)",
    tree: (justfile (assignment x (call foo y))),
  }

  test! {
    name: call_multiple_args,
    text: "x := foo(y, z)",
    tree: (justfile (assignment x (call foo y z))),
  }

  test! {
    name: call_trailing_comma,
    text: "x := foo(y,)",
    tree: (justfile (assignment x (call foo y))),
  }

  test! {
    name: recipe,
    text: "foo:",
    tree: (justfile (recipe foo)),
  }

  test! {
    name: recipe_multiple,
    text: "
      foo:
      bar:
      baz:
    ",
    tree: (justfile (recipe foo) (recipe bar) (recipe baz)),
  }

  test! {
    name: recipe_quiet,
    text: "@foo:",
    tree: (justfile (recipe #quiet foo)),
  }

  test! {
    name: recipe_parameter_single,
    text: "foo bar:",
    tree: (justfile (recipe foo (params (bar)))),
  }

  test! {
    name: recipe_parameter_multiple,
    text: "foo bar baz:",
    tree: (justfile (recipe foo (params (bar) (baz)))),
  }

  test! {
    name: recipe_default_single,
    text: r#"foo bar="baz":"#,
    tree: (justfile (recipe foo (params (bar "baz")))),
  }

  test! {
    name: recipe_default_multiple,
    text: r#"foo bar="baz" bob="biz":"#,
    tree: (justfile (recipe foo (params (bar "baz") (bob "biz")))),
  }

  test! {
    name: recipe_variadic,
    text: r#"foo +bar:"#,
    tree: (justfile (recipe foo (params +(bar)))),
  }

  test! {
    name: recipe_variadic_string_default,
    text: r#"foo +bar="baz":"#,
    tree: (justfile (recipe foo (params +(bar "baz")))),
  }

  test! {
    name: recipe_variadic_variable_default,
    text: r#"foo +bar=baz:"#,
    tree: (justfile (recipe foo (params +(bar baz)))),
  }

  test! {
    name: recipe_variadic_addition_group_default,
    text: r#"foo +bar=(baz + bob):"#,
    tree: (justfile (recipe foo (params +(bar ((+ baz bob)))))),
  }

  test! {
    name: recipe_dependency_single,
    text: "foo: bar",
    tree: (justfile (recipe foo (deps bar))),
  }

  test! {
    name: recipe_dependency_multiple,
    text: "foo: bar baz",
    tree: (justfile (recipe foo (deps bar baz))),
  }

  test! {
    name: recipe_line_single,
    text: "foo:\n bar",
    tree: (justfile (recipe foo (body ("bar")))),
  }

  test! {
    name: recipe_line_multiple,
    text: "foo:\n bar\n baz\n {{\"bob\"}}biz",
    tree: (justfile (recipe foo (body ("bar") ("baz") (("bob") "biz")))),
  }

  test! {
    name: recipe_line_interpolation,
    text: "foo:\n bar{{\"bob\"}}biz",
    tree: (justfile (recipe foo (body ("bar" ("bob") "biz")))),
  }

  test! {
    name: comment,
    text: "# foo",
    tree: (justfile),
  }

  test! {
    name: comment_alias,
    text: "alias x := y # foo",
    tree: (justfile (alias x y)),
  }

  test! {
    name: comment_assignment,
    text: "x := y # foo",
    tree: (justfile (assignment x y)),
  }

  test! {
    name: comment_export,
    text: "export x := y # foo",
    tree: (justfile (assignment #export x y)),
  }

  test! {
    name: comment_recipe,
    text: "foo: # bar",
    tree: (justfile (recipe foo)),
  }

  test! {
    name: comment_recipe_dependencies,
    text: "foo: bar # baz",
    tree: (justfile (recipe foo (deps bar))),
  }

  test! {
    name: doc_comment_single,
    text: "
      # foo
      bar:
    ",
    tree: (justfile (recipe "foo" bar)),
  }

  test! {
    name: doc_comment_recipe_clear,
    text: "
      # foo
      bar:
      baz:
    ",
    tree: (justfile (recipe "foo" bar) (recipe baz)),
  }

  test! {
    name: doc_comment_middle,
    text: "
      bar:
      # foo
      baz:
    ",
    tree: (justfile (recipe bar) (recipe "foo" baz)),
  }

  test! {
    name: doc_comment_assignment_clear,
    text: "
      # foo
      x := y
      bar:
    ",
    tree: (justfile (assignment x y) (recipe bar)),
  }

  test! {
    name: doc_comment_empty_line_clear,
    text: "
      # foo

      bar:
    ",
    tree: (justfile (recipe bar)),
  }

  test! {
    name: string_escape_tab,
    text: r#"x := "foo\tbar""#,
    tree: (justfile (assignment x "foo\tbar")),
  }

  test! {
    name: string_escape_newline,
    text: r#"x := "foo\nbar""#,
    tree: (justfile (assignment x "foo\nbar")),
  }

  test! {
    name: string_escape_carriage_return,
    text: r#"x := "foo\rbar""#,
    tree: (justfile (assignment x "foo\rbar")),
  }

  test! {
    name: string_escape_slash,
    text: r#"x := "foo\\bar""#,
    tree: (justfile (assignment x "foo\\bar")),
  }

  test! {
    name: string_escape_quote,
    text: r#"x := "foo\"bar""#,
    tree: (justfile (assignment x "foo\"bar")),
  }

  test! {
    name: recipe_variadic_with_default_after_default,
    text: r#"
      f a=b +c=d:
    "#,
    tree: (justfile (recipe f (params (a b) +(c d)))),
  }

  test! {
    name: parameter_default_concatination_variable,
    text: r#"
      x := "10"

      f y=(`echo hello` + x) +z="foo":
    "#,
    tree: (justfile
      (assignment x "10")
      (recipe f (params (y ((+ (backtick "echo hello") x))) +(z "foo")))
    ),
  }

  test! {
    name: parameter_default_multiple,
    text: r#"
      x := "10"
      f y=(`echo hello` + x) +z=("foo" + "bar"):
    "#,
    tree: (justfile
      (assignment x "10")
      (recipe f (params (y ((+ (backtick "echo hello") x))) +(z ((+ "foo" "bar")))))
    ),
  }

  test! {
    name: parse_raw_string_default,
    text: r#"

      foo a='b\t':


    "#,
    tree: (justfile (recipe foo (params (a "b\\t")))),
  }

  test! {
    name: parse_alias_after_target,
    text: r"
      foo:
        echo a
      alias f := foo
    ",
    tree: (justfile
      (recipe foo (body ("echo a")))
      (alias f foo)
    ),
  }

  test! {
    name: parse_alias_before_target,
    text: "
      alias f := foo
      foo:
        echo a
      ",
    tree: (justfile
      (alias f foo)
      (recipe foo (body ("echo a")))
    ),
  }

  test! {
    name: parse_alias_with_comment,
    text: "
      alias f := foo #comment
      foo:
        echo a
    ",
    tree: (justfile
      (alias f foo)
      (recipe foo (body ("echo a")))
    ),
  }

  test! {
    name: parse_assignment_with_comment,
    text: "
      f := foo #comment
      foo:
        echo a
    ",
    tree: (justfile
      (assignment f foo)
      (recipe foo (body ("echo a")))
    ),
  }

  test! {
    name: parse_complex,
    text: "
      x:
      y:
      z:
      foo := \"xx\"
      bar := foo
      goodbye := \"y\"
      hello a b    c   : x y    z #hello
        #! blah
        #blarg
        {{ foo + bar}}abc{{ goodbye\t  + \"x\" }}xyz
        1
        2
        3
    ",
    tree: (justfile
      (recipe x)
      (recipe y)
      (recipe z)
      (assignment foo "xx")
      (assignment bar foo)
      (assignment goodbye "y")
      (recipe hello
        (params (a) (b) (c))
        (deps x y z)
        (body
          ("#! blah")
          ("#blarg")
          (((+ foo bar)) "abc" ((+ goodbye "x")) "xyz")
          ("1")
          ("2")
          ("3")
        )
      )
    ),
  }

  test! {
    name: parse_shebang,
    text: "
      practicum := 'hello'
      install:
      \t#!/bin/sh
      \tif [[ -f {{practicum}} ]]; then
      \t\treturn
      \tfi
      ",
    tree: (justfile
      (assignment practicum "hello")
      (recipe install
        (body
         ("#!/bin/sh")
         ("if [[ -f " (practicum) " ]]; then")
         ("\treturn")
         ("fi")
        )
      )
    ),
  }

  test! {
    name: parse_simple_shebang,
    text: "a:\n #!\n  print(1)",
    tree: (justfile
      (recipe a (body ("#!") (" print(1)")))
    ),
  }

  test! {
    name: parse_assignments,
    text: r#"
      a := "0"
      c := a + b + a + b
      b := "1"
    "#,
    tree: (justfile
      (assignment a "0")
      (assignment c (+ a (+ b (+ a b))))
      (assignment b "1")
    ),
  }

  test! {
    name: parse_assignment_backticks,
    text: "
      a := `echo hello`
      c := a + b + a + b
      b := `echo goodbye`
    ",
    tree: (justfile
      (assignment a (backtick "echo hello"))
      (assignment c (+ a (+ b (+ a b))))
      (assignment b (backtick "echo goodbye"))
    ),
  }

  test! {
    name: parse_interpolation_backticks,
    text: r#"
      a:
        echo {{  `echo hello` + "blarg"   }} {{   `echo bob`   }}
    "#,
    tree: (justfile
      (recipe a
        (body ("echo " ((+ (backtick "echo hello") "blarg")) " " ((backtick "echo bob"))))
      )
    ),
  }

  test! {
    name: eof_test,
    text: "x:\ny:\nz:\na b c: x y z",
    tree: (justfile
      (recipe x)
      (recipe y)
      (recipe z)
      (recipe a (params (b) (c)) (deps x y z))
    ),
  }

  test! {
    name: string_quote_escape,
    text: r#"a := "hello\"""#,
    tree: (justfile
      (assignment a "hello\"")
    ),
  }

  test! {
    name: string_escapes,
    text: r#"a := "\n\t\r\"\\""#,
    tree: (justfile (assignment a "\n\t\r\"\\")),
  }

  test! {
    name: parameters,
    text: "
      a b c:
        {{b}} {{c}}
    ",
    tree: (justfile (recipe a (params (b) (c)) (body ((b) " " (c))))),
  }

  test! {
    name: unary_functions,
    text: "
      x := arch()

      a:
        {{os()}} {{os_family()}}
    ",
    tree: (justfile
      (assignment x (call arch))
      (recipe a (body (((call os)) " " ((call os_family)))))
    ),
  }

  test! {
    name: env_functions,
    text: r#"
      x := env_var('foo',)

      a:
        {{env_var_or_default('foo' + 'bar', 'baz',)}} {{env_var(env_var("baz"))}}
    "#,
    tree: (justfile
      (assignment x (call env_var "foo"))
      (recipe a
        (body
          (
            ((call env_var_or_default (+ "foo" "bar") "baz"))
            " "
            ((call env_var (call env_var "baz")))
          )
        )
      )
    ),
  }

  test! {
    name: parameter_default_string,
    text: r#"
      f x="abc":
    "#,
    tree: (justfile (recipe f (params (x "abc")))),
  }

  test! {
    name: parameter_default_raw_string,
    text: r"
      f x='abc':
    ",
    tree: (justfile (recipe f (params (x "abc")))),
  }

  test! {
    name: parameter_default_backtick,
    text: "
      f x=`echo hello`:
    ",
    tree: (justfile
      (recipe f (params (x (backtick "echo hello"))))
    ),
  }

  test! {
    name: parameter_default_concatination_string,
    text: r#"
      f x=(`echo hello` + "foo"):
    "#,
    tree: (justfile (recipe f (params (x ((+ (backtick "echo hello") "foo")))))),
  }

  test! {
    name: concatination_in_group,
    text: "x := ('0' + '1')",
    tree: (justfile (assignment x ((+ "0" "1")))),
  }

  test! {
    name: string_in_group,
    text: "x := ('0'   )",
    tree: (justfile (assignment x ("0"))),
  }

  test! {
    name: escaped_dos_newlines,
    text: "
      @spam:\r
      \t{ \\\r
      \t\tfiglet test; \\\r
      \t\tcargo build --color always 2>&1; \\\r
      \t\tcargo test  --color always -- --color always 2>&1; \\\r
      \t} | less\r
    ",
    tree: (justfile
      (recipe #quiet spam
        (body
         ("{ \\")
         ("\tfiglet test; \\")
         ("\tcargo build --color always 2>&1; \\")
         ("\tcargo test  --color always -- --color always 2>&1; \\")
         ("} | less")
        )
      )
    ),
  }

  test! {
    name: empty_body,
    text: "a:",
    tree: (justfile (recipe a)),
  }

  test! {
    name: single_line_body,
    text: "a:\n foo",
    tree: (justfile (recipe a (body ("foo")))),
  }

  test! {
    name: trimmed_body,
    text: "a:\n foo\n \n \n \nb:\n  ",
    tree: (justfile (recipe a (body ("foo"))) (recipe b)),
  }

  test! {
    name: set_shell_no_arguments,
    text: "set shell := ['tclsh']",
    tree: (justfile (set shell "tclsh")),
  }

  test! {
    name: set_shell_no_arguments_cooked,
    text: "set shell := [\"tclsh\"]",
    tree: (justfile (set shell "tclsh")),
  }

  test! {
    name: set_shell_no_arguments_trailing_comma,
    text: "set shell := ['tclsh',]",
    tree: (justfile (set shell "tclsh")),
  }

  test! {
    name: set_shell_with_one_argument,
    text: "set shell := ['bash', '-cu']",
    tree: (justfile (set shell "bash" "-cu")),
  }

  test! {
    name: set_shell_with_one_argument_trailing_comma,
    text: "set shell := ['bash', '-cu',]",
    tree: (justfile (set shell "bash" "-cu")),
  }

  test! {
    name: set_shell_with_two_arguments,
    text: "set shell := ['bash', '-cu', '-l']",
    tree: (justfile (set shell "bash" "-cu" "-l")),
  }

  error! {
    name: alias_syntax_multiple_rhs,
    input: "alias foo = bar baz",
    offset: 16,
    line: 0,
    column: 16,
    width: 3,
    kind: UnexpectedToken { expected: vec![Eof, Eol], found: Identifier },
  }

  error! {
    name: alias_syntax_no_rhs,
    input: "alias foo = \n",
    offset: 12,
    line: 0,
    column: 12,
    width: 1,
    kind: UnexpectedToken {expected: vec![Identifier], found:Eol},
  }

  error! {
    name:   missing_colon,
    input:  "a b c\nd e f",
    offset:  5,
    line:   0,
    column: 5,
    width:  1,
    kind:   UnexpectedToken{expected: vec![Colon, Equals, Identifier, Plus], found: Eol},
  }

  error! {
    name:   missing_default_eol,
    input:  "hello arg=\n",
    offset:  10,
    line:   0,
    column: 10,
    width:  1,
    kind:   UnexpectedToken {
      expected: vec![Backtick, Identifier, ParenL, StringCooked, StringRaw],
      found: Eol
    },
  }

  error! {
    name:   missing_default_eof,
    input:  "hello arg=",
    offset:  10,
    line:   0,
    column: 10,
    width:  0,
    kind:   UnexpectedToken {
      expected: vec![Backtick, Identifier, ParenL, StringCooked, StringRaw],
      found: Eof,
    },
  }

  error! {
    name:   missing_eol,
    input:  "a b c: z =",
    offset:  9,
    line:   0,
    column: 9,
    width:  1,
    kind:   UnexpectedToken{expected: vec![Eof, Eol, Identifier], found: Equals},
  }

  error! {
    name:   interpolation_outside_of_recipe,
    input:  "{{",
    offset:  0,
    line:   0,
    column: 0,
    width:  2,
    kind:   UnexpectedToken{expected: vec![At, Identifier], found: InterpolationStart},
  }

  error! {
    name:   unclosed_parenthesis_in_expression,
    input:  "x = foo(",
    offset: 8,
    line:   0,
    column: 8,
    width:  0,
    kind: UnexpectedToken{
      expected: vec![Backtick, Identifier, ParenL, ParenR, StringCooked, StringRaw],
      found: Eof,
    },
  }

  error! {
    name:   unclosed_parenthesis_in_interpolation,
    input:  "a:\n echo {{foo(}}",
    offset:  15,
    line:   1,
    column: 12,
    width:  2,
    kind:   UnexpectedToken{
      expected: vec![Backtick, Identifier, ParenL, ParenR, StringCooked, StringRaw],
      found: InterpolationEnd,
    },
  }

  error! {
    name:   plus_following_parameter,
    input:  "a b c+:",
    offset: 6,
    line:   0,
    column: 6,
    width:  1,
    kind:   UnexpectedToken{expected: vec![Identifier], found: Colon},
  }

  error! {
    name:   invalid_escape_sequence,
    input:  r#"foo := "\b""#,
    offset: 7,
    line:   0,
    column: 7,
    width:  4,
    kind:   InvalidEscapeSequence{character: 'b'},
  }

  error! {
    name:   bad_export,
    input:  "export a",
    offset:  8,
    line:   0,
    column: 8,
    width:  0,
    kind:   UnexpectedToken{expected: vec![Colon, Equals, Identifier, Plus], found: Eof},
  }

  error! {
    name:   parameter_follows_variadic_parameter,
    input:  "foo +a b:",
    offset: 7,
    line:   0,
    column: 7,
    width:  1,
    kind:   ParameterFollowsVariadicParameter{parameter: "b"},
  }

  error! {
    name:   parameter_after_variadic,
    input:  "foo +a bbb:",
    offset: 7,
    line:   0,
    column: 7,
    width:  3,
    kind:   ParameterFollowsVariadicParameter{parameter: "bbb"},
  }

  error! {
    name:   concatination_in_default,
    input:  "foo a=c+d e:",
    offset: 10,
    line:   0,
    column: 10,
    width:  1,
    kind:   ParameterFollowsVariadicParameter{parameter: "e"},
  }

  error! {
    name:   set_shell_empty,
    input:  "set shell := []",
    offset: 14,
    line:   0,
    column: 14,
    width:  1,
    kind:   UnexpectedToken {
      expected: vec![StringCooked, StringRaw],
      found: BracketR,
    },
  }

  error! {
    name:   set_shell_non_literal_first,
    input:  "set shell := ['bar' + 'baz']",
    offset: 20,
    line:   0,
    column: 20,
    width:  1,
    kind:   UnexpectedToken {
      expected: vec![BracketR, Comma],
      found: Plus,
    },
  }

  error! {
    name:   set_shell_non_literal_second,
    input:  "set shell := ['biz', 'bar' + 'baz']",
    offset: 27,
    line:   0,
    column: 27,
    width:  1,
    kind:   UnexpectedToken {
      expected: vec![BracketR, Comma],
      found: Plus,
    },
  }

  error! {
    name:   set_shell_bad_comma,
    input:  "set shell := ['bash',",
    offset: 21,
    line:   0,
    column: 21,
    width:  0,
    kind:   UnexpectedToken {
      expected: vec![BracketR, StringCooked, StringRaw],
      found: Eof,
    },
  }

  error! {
    name:   set_shell_bad,
    input:  "set shell := ['bash'",
    offset: 20,
    line:   0,
    column: 20,
    width:  0,
    kind:   UnexpectedToken {
      expected: vec![BracketR, Comma],
      found: Eof,
    },
  }

  error! {
    name:   set_unknown,
    input:  "set shall := []",
    offset: 4,
    line:   0,
    column: 4,
    width:  5,
    kind:   UnknownSetting {
      setting: "shall",
    },
  }

  error! {
    name:   set_shell_non_string,
    input:  "set shall := []",
    offset: 4,
    line:   0,
    column: 4,
    width:  5,
    kind:   UnknownSetting {
      setting: "shall",
    },
  }
}
