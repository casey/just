use {super::*, TokenKind::*};

/// Just language parser
///
/// The parser is a (hopefully) straightforward recursive descent parser.
///
/// It uses a few tokens of lookahead to disambiguate different constructs.
///
/// The `expect_*` and `presume_*` methods are similar in that they assert the
/// type of unparsed tokens and consume them. However, upon encountering an
/// unexpected token, the `expect_*` methods return an unexpected token error,
/// whereas the `presume_*` tokens return an internal error.
///
/// The `presume_*` methods are used when the token stream has been inspected in
/// some other way, and thus encountering an unexpected token is a bug in Just,
/// and not a syntax error.
///
/// All methods starting with `parse_*` parse and return a language construct.
///
/// The parser tracks an expected set of tokens as it parses. This set contains
/// all tokens which would have been accepted at the current point in the
/// parse. Whenever the parser tests for a token that would be accepted, but
/// does not find it, it adds that token to the set. When the parser accepts a
/// token, the set is cleared. If the parser finds a token which is unexpected,
/// the elements of the set are printed in the resultant error message.
pub(crate) struct Parser<'run, 'src> {
  expected_tokens: BTreeSet<TokenKind>,
  file_depth: u32,
  import_offsets: Vec<usize>,
  items: Vec<Item<'src>>,
  list_features: Vec<(ListFeature, Token<'src>)>,
  module_namepath: Option<&'run Namepath<'src>>,
  next_token: usize,
  numerator: &'run mut Numerator,
  recursion_depth: usize,
  tokens: &'run [Token<'src>],
  unstable_features: BTreeSet<UnstableFeature>,
  working_directory: &'run Path,
}

impl<'run, 'src> Parser<'run, 'src> {
  /// Parse `tokens` into an `Ast`
  pub(crate) fn parse(
    file_depth: u32,
    import_offsets: &[usize],
    module_namepath: Option<&'run Namepath<'src>>,
    numerator: &'run mut Numerator,
    tokens: &'run [Token<'src>],
    working_directory: &'run Path,
  ) -> CompileResult<'src, Ast<'src>> {
    Self {
      expected_tokens: BTreeSet::new(),
      file_depth,
      import_offsets: import_offsets.to_vec(),
      items: Vec::new(),
      list_features: Vec::new(),
      module_namepath,
      next_token: 0,
      numerator,
      recursion_depth: 0,
      tokens,
      unstable_features: BTreeSet::new(),
      working_directory,
    }
    .parse_ast()
  }

  pub(crate) fn parse_source(
    numerator: &mut Numerator,
    path: &'src Path,
    source: &Source<'src>,
    src: &'src str,
  ) -> CompileResult<'src, Ast<'src>> {
    let tokens = Lexer::lex(path, src)?;

    Parser::parse(
      source.file_depth,
      &source.import_offsets,
      source.namepath.as_ref(),
      numerator,
      &tokens,
      &source.working_directory,
    )
  }

  #[cfg(test)]
  pub(crate) fn parse_tokens(
    numerator: &'run mut Numerator,
    tokens: &'run [Token<'src>],
  ) -> CompileResult<'src, Ast<'src>> {
    Self::parse(0, &[], None, numerator, tokens, "".as_ref())
  }

  fn error(&self, kind: CompileErrorKind<'src>) -> CompileResult<'src, CompileError<'src>> {
    Ok(self.next()?.error(kind))
  }

  fn list_feature(&mut self, feature: ListFeature, token: Token<'src>) {
    self.list_features.push((feature, token));
  }

  /// Construct an unexpected token error with the token returned by
  /// `Parser::next`
  fn unexpected_token(&self) -> CompileResult<'src, CompileError<'src>> {
    self.error(CompileErrorKind::UnexpectedToken {
      expected: self
        .expected_tokens
        .iter()
        .copied()
        .filter(|kind| *kind != ByteOrderMark)
        .collect::<Vec<TokenKind>>(),
      found: self.next()?.kind,
    })
  }

  fn internal_error(&self, message: impl Into<String>) -> CompileResult<'src, CompileError<'src>> {
    self.error(CompileErrorKind::Internal {
      message: message.into(),
    })
  }

  /// An iterator over the remaining significant tokens
  fn rest(&self) -> impl Iterator<Item = Token<'src>> + 'run {
    self.tokens[self.next_token..]
      .iter()
      .copied()
      .filter(|token| token.kind != Whitespace)
  }

  /// The next significant token
  fn next(&self) -> CompileResult<'src, Token<'src>> {
    if let Some(token) = self.rest().next() {
      Ok(token)
    } else {
      Err(self.internal_error("`Parser::next()` called after end of token stream")?)
    }
  }

  /// Check if the next significant token is of kind `kind`
  fn next_is(&mut self, kind: TokenKind) -> bool {
    self.next_are(&[kind])
  }

  /// Check if the next significant token is `keyword`
  fn next_is_keyword(&self, keyword: Keyword) -> bool {
    self
      .rest()
      .next()
      .is_some_and(|token| token.kind == Identifier && token.lexeme() == keyword.lexeme())
  }

  /// Check if the next significant tokens are of kinds `kinds`
  ///
  /// The first token in `kinds` will be added to the expected token set.
  fn next_are(&mut self, kinds: &[TokenKind]) -> bool {
    if let Some(&kind) = kinds.first() {
      self.expected_tokens.insert(kind);
    }

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

  /// Check if the next significant tokens are of kinds `kinds`, followed by a
  /// comment, end-of-file, or end-of-line.
  ///
  /// The first token in `kinds` will be added to the expected token set.
  fn line_is(&mut self, kinds: &[TokenKind]) -> bool {
    if let Some(&kind) = kinds.first() {
      self.expected_tokens.insert(kind);
    }

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

    if let Some(token) = rest.next()
      && matches!(token.kind, Comment | Eof | Eol)
    {
      return true;
    }

    false
  }

  /// Advance past one significant token, clearing the expected token set.
  fn advance(&mut self) -> CompileResult<'src, Token<'src>> {
    self.expected_tokens.clear();

    for skipped in &self.tokens[self.next_token..] {
      self.next_token += 1;

      if skipped.kind != Whitespace {
        return Ok(*skipped);
      }
    }

    Err(self.internal_error("`Parser::advance()` advanced past end of token stream")?)
  }

  /// Return next token if it is of kind `expected`, otherwise, return an
  /// unexpected token error
  fn expect(&mut self, expected: TokenKind) -> CompileResult<'src, Token<'src>> {
    if let Some(token) = self.accept(expected)? {
      Ok(token)
    } else {
      Err(self.unexpected_token()?)
    }
  }

  /// Return the next token if it is any of kinds in `expected`, otherwise,
  /// return an unexpected token error
  fn expect_any(&mut self, expected: &[TokenKind]) -> CompileResult<'src, Token<'src>> {
    for &kind in expected {
      if let Some(token) = self.accept(kind)? {
        return Ok(token);
      }
    }

    Err(self.unexpected_token()?)
  }

  /// Return an unexpected token error if the next token is not an EOL
  fn expect_eol(&mut self) -> CompileResult<'src> {
    self.accept(Comment)?;

    if self.next_is(Eof) {
      return Ok(());
    }

    self.expect(Eol).map(|_| ())
  }

  fn expect_keyword(&mut self, expected: Keyword) -> CompileResult<'src> {
    let found = self.advance()?;

    if found.kind == Identifier && expected == found.lexeme() {
      Ok(())
    } else {
      Err(found.error(CompileErrorKind::ExpectedKeyword {
        expected: vec![expected],
        found,
      }))
    }
  }

  /// Return an internal error if the next token is not of kind `Identifier`
  /// with lexeme `keyword`.
  fn presume_keyword(&mut self, keyword: Keyword) -> CompileResult<'src, Token<'src>> {
    let next = self.advance()?;

    if next.kind != Identifier {
      Err(self.internal_error(format!(
        "presumed next token would have kind {Identifier}, but found {}",
        next.kind
      ))?)
    } else if keyword == next.lexeme() {
      Ok(next)
    } else {
      Err(self.internal_error(format!(
        "presumed next token would have lexeme \"{keyword}\", but found \"{}\"",
        next.lexeme(),
      ))?)
    }
  }

  /// Return an internal error if the next token is not of kind `kind`.
  fn presume(&mut self, kind: TokenKind) -> CompileResult<'src, Token<'src>> {
    let next = self.advance()?;

    if next.kind == kind {
      Ok(next)
    } else {
      Err(self.internal_error(format!(
        "presumed next token would have kind {kind:?}, but found {:?}",
        next.kind
      ))?)
    }
  }

  /// Return an internal error if the next token is not one of kinds `kinds`.
  fn presume_any(&mut self, kinds: &[TokenKind]) -> CompileResult<'src, Token<'src>> {
    let next = self.advance()?;
    if kinds.contains(&next.kind) {
      Ok(next)
    } else {
      Err(self.internal_error(format!(
        "presumed next token would be {}, but found {}",
        List::or(kinds),
        next.kind
      ))?)
    }
  }

  /// Accept and return a token of kind `kind`
  fn accept(&mut self, kind: TokenKind) -> CompileResult<'src, Option<Token<'src>>> {
    if self.next_is(kind) {
      Ok(Some(self.advance()?))
    } else {
      Ok(None)
    }
  }

  /// Return an error if the next token is of kind `forbidden`
  fn forbid<F>(&self, forbidden: TokenKind, error: F) -> CompileResult<'src>
  where
    F: FnOnce(Token) -> CompileError,
  {
    let next = self.next()?;

    if next.kind == forbidden {
      Err(error(next))
    } else {
      Ok(())
    }
  }

  /// Accept a double-colon separated sequence of identifiers
  fn accept_namepath(&mut self) -> CompileResult<'src, Option<Namepath<'src>>> {
    if self.next_is(Identifier) {
      Ok(Some(self.parse_namepath()?))
    } else {
      Ok(None)
    }
  }

  fn accept_keyword(&mut self, keyword: Keyword) -> CompileResult<'src, Option<Name<'src>>> {
    let next = self.next()?;

    if next.kind == Identifier && next.lexeme() == keyword.lexeme() {
      self.advance()?;
      Ok(Some(Name::from_identifier(next)))
    } else {
      Ok(None)
    }
  }

  fn accepted_keyword(&mut self, keyword: Keyword) -> CompileResult<'src, bool> {
    Ok(self.accept_keyword(keyword)?.is_some())
  }

  /// Accept a dependency
  fn accept_dependency(&mut self) -> CompileResult<'src, Option<UnresolvedDependency<'src>>> {
    if let Some(recipe) = self.accept_namepath()? {
      Ok(Some(UnresolvedDependency {
        arguments: Vec::new(),
        recipe,
      }))
    } else if self.next_is(Asterisk) || self.next_is(ParenL) {
      let star = self.accept(Asterisk)?;

      self.expect(ParenL)?;

      let recipe = self.parse_namepath()?;

      let mut arguments = Vec::new();
      let mut starred_argument = None;

      while !self.accepted(ParenR)? {
        let token = self.accept(Asterisk)?;

        if let Some(token) = token {
          if starred_argument.is_some() {
            return Err(token.error(CompileErrorKind::MappedDependencyMultipleStarredArguments));
          }
          starred_argument = Some(token);
        }

        let expression = if token.is_some() {
          self.parse_value()?
        } else {
          self.parse_expression()?
        };

        arguments.push(DependencyArgument {
          expression,
          starred: token.is_some(),
        });
      }

      match (star, starred_argument) {
        (None, None) | (Some(_), Some(_)) => {}
        (Some(star), None) => {
          return Err(star.error(CompileErrorKind::MappedDependencyWithoutStarredArgument));
        }
        (None, Some(starred)) => {
          return Err(starred.error(CompileErrorKind::StarredArgumentOutsideMappedDependency));
        }
      }

      Ok(Some(UnresolvedDependency { arguments, recipe }))
    } else {
      Ok(None)
    }
  }

  /// Accept and return `true` if next token is of kind `kind`
  fn accepted(&mut self, kind: TokenKind) -> CompileResult<'src, bool> {
    Ok(self.accept(kind)?.is_some())
  }

  fn take_doc_comment(&mut self, attributes: &AttributeSet<'src>) -> Option<String> {
    if attributes.contains(AttributeKind::Doc) {
      return None;
    }

    let mut items = self.items.iter().rev();

    if !matches!(items.next()?, Item::Newline) {
      return None;
    }

    let Item::Comment(contents) = items.next()? else {
      return None;
    };

    let first = match items.next() {
      None => true,
      Some(Item::Newline) => false,
      Some(_) => return None,
    };

    if first && contents.starts_with("#!") {
      return None;
    }

    let doc = contents[1..].trim_start().into();

    self.items.pop().unwrap();
    self.items.pop().unwrap();

    Some(doc)
  }

  /// Parse a justfile, consumes self
  fn parse_ast(mut self) -> CompileResult<'src, Ast<'src>> {
    self.accept(ByteOrderMark)?;

    while !self.accepted(Eof)? {
      let mut attributes = self.parse_attributes()?;

      let item = self.parse_item(&mut attributes)?;
      self.items.push(item);

      let unterminated = match self.items.last().unwrap() {
        Item::Newline => false,
        Item::Recipe(recipe) => {
          if recipe.body.is_empty() {
            true
          } else {
            self.items.push(Item::Newline);
            false
          }
        }
        _ => true,
      };

      if let Some((token, attributes)) = attributes {
        return Err(token.error(CompileErrorKind::ExtraneousAttributes {
          count: attributes.len(),
        }));
      }

      if unterminated {
        if let Some(comment) = self.accept(Comment)? {
          self.items.push(Item::Comment(comment.lexeme().trim_end()));
        }

        if !self.next_is(Eof) {
          self.expect(Eol)?;
          self.items.push(Item::Newline);
        }
      }
    }

    if self.next_token != self.tokens.len() {
      return Err(self.internal_error(format!(
        "parse completed with {} unparsed tokens",
        self.tokens.len() - self.next_token,
      ))?);
    }

    Ok(Ast {
      items: self.items,
      list_features: self.list_features,
      module_path: self.module_namepath.map(Into::into).unwrap_or_default(),
      unstable_features: self.unstable_features,
      warnings: Vec::new(),
      working_directory: self.working_directory.into(),
    })
  }

  fn parse_item(
    &mut self,
    attributes: &mut Option<(Token<'src>, AttributeSet<'src>)>,
  ) -> CompileResult<'src, Item<'src>> {
    let mut take_attributes = || {
      attributes
        .take()
        .map(|(_token, attributes)| attributes)
        .unwrap_or_default()
    };

    let item = if let Some(comment) = self.accept(Comment)? {
      Item::Comment(comment.lexeme().trim_end())
    } else if self.accepted(Eol)? {
      Item::Newline
    } else if self.next_is(Identifier) {
      let next = self.next()?;
      match Keyword::from_lexeme(next.lexeme()) {
        Some(Keyword::Alias) if self.next_are(&[Identifier, Identifier, ColonEquals]) => {
          Item::Alias(self.parse_alias(take_attributes())?)
        }
        Some(Keyword::Eager) if self.next_are(&[Identifier, Identifier, ColonEquals]) => {
          self.presume_keyword(Keyword::Eager)?;
          Item::Assignment(self.parse_assignment(take_attributes(), true, false)?)
        }
        Some(Keyword::Export) if self.next_are(&[Identifier, Identifier, ColonEquals]) => {
          self.presume_keyword(Keyword::Export)?;
          Item::Assignment(self.parse_assignment(take_attributes(), false, true)?)
        }
        Some(Keyword::Unexport) if self.line_is(&[Identifier, Identifier]) => {
          self.presume_keyword(Keyword::Unexport)?;
          let name = self.parse_name()?;
          let attributes = take_attributes();
          attributes.ensure_valid_attributes(ItemKind::Unexport, *name)?;
          Item::Unexport { attributes, name }
        }
        Some(Keyword::Import)
          if self.next_are(&[Identifier, Identifier, StringToken])
            || self.next_are(&[Identifier, QuestionMark])
            || self.next_are(&[Identifier, StringToken]) =>
        {
          self.presume_keyword(Keyword::Import)?;
          let optional = self.accepted(QuestionMark)?;
          let relative = self.parse_string_literal()?;
          let attributes = take_attributes();
          attributes.ensure_valid_attributes(ItemKind::Import, relative.token)?;
          Item::Import {
            absolute: None,
            attributes,
            optional,
            relative,
          }
        }
        Some(Keyword::Mod)
          if self.line_is(&[Identifier, Identifier])
            || self.next_are(&[Identifier, Identifier, Identifier, StringToken])
            || self.next_are(&[Identifier, Identifier, StringToken])
            || self.next_are(&[Identifier, QuestionMark]) =>
        {
          self.presume_keyword(Keyword::Mod)?;

          let optional = self.accepted(QuestionMark)?;

          let name = self.parse_name()?;

          let relative = if self.next_is(StringToken) || self.next_are(&[Identifier, StringToken]) {
            Some(self.parse_string_literal()?)
          } else {
            None
          };

          let attributes = take_attributes();

          attributes.ensure_valid_attributes(ItemKind::Module, *name)?;

          let doc = self.take_doc_comment(&attributes);

          Item::Module {
            absolute: None,
            attributes,
            doc,
            name,
            optional,
            relative,
          }
        }
        Some(Keyword::Set)
          if self.next_are(&[Identifier, Identifier, ColonEquals])
            || self.line_is(&[Identifier, Identifier]) =>
        {
          Item::Setting(self.parse_set(take_attributes())?)
        }
        _ => {
          if self.next_are(&[Identifier, ParenL]) {
            Item::Function(self.parse_function_definition(take_attributes())?)
          } else if self.next_are(&[Identifier, ColonEquals]) {
            Item::Assignment(self.parse_assignment(take_attributes(), false, false)?)
          } else {
            Item::Recipe(self.parse_recipe(take_attributes(), false)?)
          }
        }
      }
    } else if self.accepted(At)? {
      Item::Recipe(self.parse_recipe(take_attributes(), true)?)
    } else {
      return Err(self.unexpected_token()?);
    };

    Ok(item)
  }

  /// Parse an alias, e.g `alias name := target`
  fn parse_alias(&mut self, attributes: AttributeSet<'src>) -> CompileResult<'src, Alias<'src>> {
    self.presume_keyword(Keyword::Alias)?;
    let name = self.parse_name()?;
    self.presume_any(&[Equals, ColonEquals])?;
    let target = self.parse_namepath()?;

    attributes.ensure_valid_attributes(ItemKind::Alias, *name)?;

    Ok(Alias {
      attributes,
      name,
      target,
    })
  }

  fn parse_function_definition(
    &mut self,
    attributes: AttributeSet<'src>,
  ) -> CompileResult<'src, FunctionDefinition<'src>> {
    self
      .unstable_features
      .insert(UnstableFeature::UserDefinedFunctions);

    let name = self.parse_name()?;

    attributes.ensure_valid_attributes(ItemKind::Function, *name)?;

    self.presume(ParenL)?;

    let mut parameters = Vec::new();
    while !self.next_is(ParenR) {
      parameters.push((self.parse_name()?, self.numerator.next_binding()));
      if !self.accepted(Comma)? {
        break;
      }
    }

    self.expect(ParenR)?;

    self.expect(ColonEquals)?;

    let body = self.parse_expression()?;

    Ok(FunctionDefinition {
      attributes,
      body,
      name,
      parameters,
    })
  }

  /// Parse an assignment, e.g. `foo := bar`
  fn parse_assignment(
    &mut self,
    attributes: AttributeSet<'src>,
    eager: bool,
    export: bool,
  ) -> CompileResult<'src, Assignment<'src>> {
    let name = self.parse_name()?;
    self.presume(ColonEquals)?;
    let value = self.parse_expression()?;

    attributes.ensure_valid_attributes(ItemKind::Assignment, *name)?;

    let private = attributes.private();

    Ok(Assignment {
      attributes,
      eager,
      export,
      file_depth: self.file_depth,
      name,
      number: self.numerator.next_binding(),
      prelude: false,
      private: private || name.lexeme().starts_with('_'),
      value,
    })
  }

  /// Parse an expression, e.g. `1 + 2`
  fn parse_expression(&mut self) -> CompileResult<'src, Expression<'src>> {
    self.parse_expression_with_condition(false)
  }

  fn parse_expression_with_condition(
    &mut self,
    condition: bool,
  ) -> CompileResult<'src, Expression<'src>> {
    if self.recursion_depth == RECURSION_LIMIT {
      let token = self.next()?;
      return Err(CompileError::new(
        token,
        CompileErrorKind::ParsingRecursionDepthExceeded,
      ));
    }

    self.recursion_depth += 1;

    let disjunct = self.parse_disjunct(condition)?;

    let expression = if let Some(token) = self.accept(BarBar)? {
      self.list_feature(ListFeature::LogicalOperator, token);
      let lhs = disjunct.into();
      let rhs = self.parse_expression_with_condition(false)?.into();
      Expression::Or { lhs, rhs }
    } else {
      disjunct
    };

    self.recursion_depth -= 1;

    Ok(expression)
  }

  fn parse_disjunct(&mut self, condition: bool) -> CompileResult<'src, Expression<'src>> {
    let conjunct = self.parse_comparison(condition)?;

    let expression = if let Some(token) = self.accept(AmpersandAmpersand)? {
      self.list_feature(ListFeature::LogicalOperator, token);
      let lhs = conjunct.into();
      let rhs = self.parse_disjunct(false)?.into();
      Expression::And { lhs, rhs }
    } else {
      conjunct
    };

    Ok(expression)
  }

  fn parse_comparison(&mut self, condition: bool) -> CompileResult<'src, Expression<'src>> {
    let lhs = self.parse_conjunct()?;

    let (token, operator) = if let Some(token) = self.accept(BangEquals)? {
      (token, ConditionalOperator::Inequality)
    } else if let Some(token) = self.accept(EqualsTilde)? {
      (token, ConditionalOperator::RegexMatch)
    } else if let Some(token) = self.accept(BangTilde)? {
      (token, ConditionalOperator::RegexMismatch)
    } else if let Some(token) = self.accept(EqualsEquals)? {
      (token, ConditionalOperator::Equality)
    } else {
      return Ok(lhs);
    };

    if !condition {
      self.list_feature(ListFeature::ComparisonOperator, token);
    }

    let rhs = self.parse_conjunct()?;

    Ok(Expression::Comparison {
      lhs: lhs.into(),
      operator,
      rhs: rhs.into(),
      token,
    })
  }

  fn parse_conjunct(&mut self) -> CompileResult<'src, Expression<'src>> {
    if self.next_is_keyword(Keyword::If) {
      self.parse_conditional()
    } else if let Some(operator) = self.accept(Slash)? {
      let lhs = None;
      let rhs = self.parse_conjunct()?.into();
      Ok(Expression::Join { lhs, operator, rhs })
    } else {
      let value = self.parse_value()?;

      if let Some(operator) = self.accept(Slash)? {
        let lhs = Some(Box::new(value));
        let rhs = self.parse_conjunct()?.into();
        Ok(Expression::Join { lhs, operator, rhs })
      } else if let Some(operator) = self.accept(PlusPlus)? {
        self.list_feature(ListFeature::ListConcatenationOperator, operator);
        let lhs = value.into();
        let rhs = self.parse_conjunct()?.into();
        Ok(Expression::ListConcatenation { lhs, operator, rhs })
      } else if let Some(operator) = self.accept(Plus)? {
        let lhs = value.into();
        let rhs = self.parse_conjunct()?.into();
        Ok(Expression::Concatenation { lhs, operator, rhs })
      } else {
        Ok(value)
      }
    }
  }

  /// Parse a conditional, e.g. `if a == b { "foo" } else { "bar" }`
  fn parse_conditional(&mut self) -> CompileResult<'src, Expression<'src>> {
    let if_token = self.presume_keyword(Keyword::If)?;

    let condition = self.parse_condition()?;

    self.expect(BraceL)?;

    let then = self.parse_expression()?;

    self.expect(BraceR)?;

    let otherwise = if self.accepted_keyword(Keyword::Else)? {
      if self.next_is_keyword(Keyword::If) {
        Some(self.parse_conditional()?.into())
      } else {
        self.expect(BraceL)?;
        let otherwise = self.parse_expression()?;
        self.expect(BraceR)?;
        Some(otherwise.into())
      }
    } else {
      self.list_feature(ListFeature::IfWithoutElse, if_token);
      None
    };

    Ok(Expression::Conditional {
      condition: condition.into(),
      then: then.into(),
      otherwise,
    })
  }

  /// Parse the condition of an `if` or `assert`
  fn parse_condition(&mut self) -> CompileResult<'src, Expression<'src>> {
    let token = self.next()?;
    let condition = self.parse_expression_with_condition(true)?;
    if !matches!(condition, Expression::Comparison { .. }) {
      self.list_feature(ListFeature::NonComparisonCondition, token);
    }
    Ok(condition)
  }

  fn parse_format_string(&mut self) -> CompileResult<'src, Expression<'src>> {
    self.presume_keyword(Keyword::F)?;

    let start = self.parse_string_literal_in_state(StringState::FormatStart)?;

    let kind = StringKind::from_string_or_backtick(start.token)?;

    let mut more = start.token.kind == FormatStringStart;

    let mut expressions = Vec::new();

    while more {
      let expression = self.parse_expression()?;
      more = self.next_is(FormatStringContinue);
      expressions.push((
        expression,
        self.parse_string_literal_in_state(StringState::FormatContinue(kind))?,
      ));
    }

    Ok(Expression::FormatString { start, expressions })
  }

  // Check if the next tokens are a shell-expanded string, i.e., `x"foo"`.
  //
  // This function skips initial whitespace tokens, but thereafter is
  // whitespace-sensitive, so `x"foo"` is a shell-expanded string, whereas `x
  // "foo"` is not.
  fn next_is_shell_expanded_string(&self) -> bool {
    let mut tokens = self
      .tokens
      .iter()
      .skip(self.next_token)
      .skip_while(|token| token.kind == Whitespace);

    tokens
      .next()
      .is_some_and(|token| token.kind == Identifier && token.lexeme() == Keyword::X.lexeme())
      && tokens.next().is_some_and(|token| token.kind == StringToken)
  }

  // Check if the next tokens are a format string, i.e., `f"foo"`.
  //
  // This function skips initial whitespace tokens, but thereafter is
  // whitespace-sensitive, so `f"foo"` is a format string, whereas `f
  // "foo"` is not.
  fn next_is_format_string(&self) -> bool {
    let mut tokens = self
      .tokens
      .iter()
      .skip(self.next_token)
      .skip_while(|token| token.kind == Whitespace);

    tokens
      .next()
      .is_some_and(|token| token.kind == Identifier && token.lexeme() == Keyword::F.lexeme())
      && tokens
        .next()
        .is_some_and(|token| matches!(token.kind, StringToken | FormatStringStart))
  }

  /// Parse a value, e.g. `(bar)`
  fn parse_value(&mut self) -> CompileResult<'src, Expression<'src>> {
    if let Some(token) = self.accept(Bang)? {
      self.list_feature(ListFeature::NegationOperator, token);
      Ok(Expression::Not {
        operand: self.parse_value()?.into(),
      })
    } else if self.next_is(StringToken) || self.next_is_shell_expanded_string() {
      Ok(Expression::StringLiteral {
        string_literal: self.parse_string_literal()?,
      })
    } else if self.next_is_format_string() {
      self.parse_format_string()
    } else if self.next_is(Backtick) {
      let next = self.next()?;
      let kind = StringKind::from_string_or_backtick(next)?;
      let contents =
        &next.lexeme()[kind.delimiter_len()..next.lexeme().len() - kind.delimiter_len()];
      let token = self.advance()?;
      let contents = if kind.indented() {
        unindent(contents)
      } else {
        contents.to_owned()
      };

      if contents.starts_with("#!") {
        return Err(next.error(CompileErrorKind::BacktickShebang));
      }
      Ok(Expression::Backtick { contents, token })
    } else if self.next_is(Identifier) {
      if let Some(name) = self.accept_keyword(Keyword::Assert)? {
        self.expect(ParenL)?;
        let condition = Box::new(self.parse_condition()?);
        let message = if self.accepted(Comma)? {
          Some(Box::new(self.parse_expression()?))
        } else {
          None
        };
        self.expect(ParenR)?;
        Ok(Expression::Assert {
          condition,
          message,
          name,
        })
      } else {
        let name = self.parse_name()?;

        if self.next_is(ParenL) {
          let arguments = self.parse_sequence()?;
          match name.lexeme() {
            "bool" => {
              self.list_feature(ListFeature::BoolFunction, *name);
            }
            "join_list" => {
              self.list_feature(ListFeature::JoinListFunction, *name);
            }
            "num_jobs" => {
              self.list_feature(ListFeature::NumJobsFunction, *name);
            }
            "show" => {
              self.list_feature(ListFeature::ShowFunction, *name);
            }
            "split" => {
              self.list_feature(ListFeature::SplitFunction, *name);
            }
            "which" => {
              self.list_feature(ListFeature::WhichFunction, *name);
            }
            _ => {}
          }
          Ok(Expression::Call { name, arguments })
        } else {
          Ok(Expression::Variable { name })
        }
      }
    } else if self.next_is(ParenL) {
      self.presume(ParenL)?;
      let contents = self.parse_expression()?.into();
      self.expect(ParenR)?;
      Ok(Expression::Group { contents })
    } else if self.next_is(BracketL) {
      self.parse_list()
    } else {
      Err(self.unexpected_token()?)
    }
  }

  /// Parse a list literal, e.g. `[a, b, c]`
  fn parse_list(&mut self) -> CompileResult<'src, Expression<'src>> {
    let bracket = self.presume(BracketL)?;

    self.list_feature(ListFeature::ListLiteral, bracket);

    let mut elements = Vec::new();

    while !self.next_is(BracketR) {
      elements.push(self.parse_expression()?);

      if !self.accepted(Comma)? {
        break;
      }
    }

    self.expect(BracketR)?;

    Ok(Expression::List {
      elements,
      open: bracket,
    })
  }

  /// Parse a string literal, e.g. `"FOO"`
  fn parse_string_literal(&mut self) -> CompileResult<'src, StringLiteral<'src>> {
    self.parse_string_literal_in_state(StringState::Normal)
  }

  /// Parse a string literal, e.g. `"FOO"`
  fn parse_string_literal_in_state(
    &mut self,
    state: StringState,
  ) -> CompileResult<'src, StringLiteral<'src>> {
    let expand = if matches!(state, StringState::Normal) && self.next_is(Identifier) {
      self.expect_keyword(Keyword::X)?;
      true
    } else {
      false
    };

    let token = match state {
      StringState::Normal => self.expect(StringToken)?,
      StringState::FormatStart => self.expect_any(&[StringToken, FormatStringStart])?,
      StringState::FormatContinue(_) => {
        self.expect_any(&[FormatStringContinue, FormatStringEnd])?
      }
    };

    let kind = match state {
      StringState::Normal | StringState::FormatStart => StringKind::from_string_or_backtick(token)?,
      StringState::FormatContinue(kind) => kind,
    };

    let open = if matches!(token.kind, FormatStringContinue | FormatStringEnd) {
      Lexer::INTERPOLATION_END.len()
    } else {
      kind.delimiter_len()
    };

    let close = if matches!(token.kind, FormatStringStart | FormatStringContinue) {
      Lexer::INTERPOLATION_START.len()
    } else {
      kind.delimiter_len()
    };

    let raw = &token.lexeme()[open..token.lexeme().len() - close];

    let unindented = if kind.indented() && matches!(token.kind, StringToken) {
      unindent(raw)
    } else {
      raw.to_owned()
    };

    let undelimited = if matches!(state, StringState::Normal) {
      unindented
    } else {
      unindented.replace(Lexer::INTERPOLATION_ESCAPE, Lexer::INTERPOLATION_START)
    };

    let cooked = if kind.processes_escape_sequences() {
      Self::cook_string(token, &undelimited)?
    } else {
      undelimited
    };

    let cooked = if expand {
      shellexpand::full(&cooked)
        .map_err(|err| token.error(CompileErrorKind::ShellExpansion { err }))?
        .into_owned()
    } else {
      cooked
    };

    Ok(StringLiteral {
      token,
      cooked,
      expand,
      kind,
      part: match token.kind {
        FormatStringStart => Some(FormatStringPart::Start),
        FormatStringContinue => Some(FormatStringPart::Continue),
        FormatStringEnd => Some(FormatStringPart::End),
        StringToken => {
          if matches!(state, StringState::Normal) {
            None
          } else {
            Some(FormatStringPart::Single)
          }
        }
        _ => {
          return Err(token.error(CompileErrorKind::Internal {
            message: "unexpected token kind while parsing string literal".into(),
          }));
        }
      },
    })
  }

  // Transform escape sequences in from string literal `token` with content `text`
  fn cook_string(token: Token<'src>, text: &str) -> CompileResult<'src, String> {
    #[derive(PartialEq, Eq)]
    enum State {
      Backslash,
      BackslashCarriageReturn,
      Initial,
      Unicode,
      UnicodeValue { hex: String },
    }

    let mut cooked = String::new();

    let mut state = State::Initial;

    for c in text.chars() {
      match state {
        State::Initial => {
          if c == '\\' {
            state = State::Backslash;
          } else {
            cooked.push(c);
          }
        }
        State::Backslash if c == 'u' => {
          state = State::Unicode;
        }
        State::Backslash => {
          state = State::Initial;
          match c {
            'n' => cooked.push('\n'),
            'r' => cooked.push('\r'),
            't' => cooked.push('\t'),
            '\\' => cooked.push('\\'),
            '\n' => {}
            '\r' => state = State::BackslashCarriageReturn,
            '"' => cooked.push('"'),
            character => {
              return Err(token.error(CompileErrorKind::InvalidEscapeSequence { character }));
            }
          }
        }
        State::BackslashCarriageReturn => match c {
          '\n' => state = State::Initial,
          _ => {
            return Err(token.error(CompileErrorKind::InvalidEscapeSequence { character: '\r' }));
          }
        },
        State::Unicode => match c {
          '{' => {
            state = State::UnicodeValue { hex: String::new() };
          }
          character => {
            return Err(token.error(CompileErrorKind::UnicodeEscapeDelimiter { character }));
          }
        },
        State::UnicodeValue { ref mut hex } => match c {
          '}' => {
            if hex.is_empty() {
              return Err(token.error(CompileErrorKind::UnicodeEscapeEmpty));
            }

            let codepoint = u32::from_str_radix(hex, 16).unwrap();

            cooked.push(char::from_u32(codepoint).ok_or_else(|| {
              token.error(CompileErrorKind::UnicodeEscapeRange { hex: hex.clone() })
            })?);

            state = State::Initial;
          }
          '0'..='9' | 'A'..='F' | 'a'..='f' => {
            hex.push(c);
            if hex.len() > 6 {
              return Err(token.error(CompileErrorKind::UnicodeEscapeLength { hex: hex.clone() }));
            }
          }
          _ => {
            return Err(token.error(CompileErrorKind::UnicodeEscapeCharacter { character: c }));
          }
        },
      }
    }

    match state {
      State::Initial => Ok(cooked),
      State::BackslashCarriageReturn => {
        Err(token.error(CompileErrorKind::InvalidEscapeSequence { character: '\r' }))
      }
      _ => Err(token.error(CompileErrorKind::UnicodeEscapeUnterminated)),
    }
  }

  /// Parse a name from an identifier token
  fn parse_name(&mut self) -> CompileResult<'src, Name<'src>> {
    self.expect(Identifier).map(Name::from_identifier)
  }

  /// Parse a path of `::` separated names
  fn parse_namepath(&mut self) -> CompileResult<'src, Namepath<'src>> {
    let first = self.parse_name()?;
    let mut path = Namepath::from(first);

    while self.accepted(ColonColon)? {
      let name = self.parse_name()?;
      path.push(name);
    }

    Ok(path)
  }

  /// Parse sequence of comma-separated expressions
  fn parse_sequence(&mut self) -> CompileResult<'src, Vec<Expression<'src>>> {
    self.presume(ParenL)?;

    let mut elements = Vec::new();

    while !self.next_is(ParenR) {
      elements.push(self.parse_expression()?);

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
    attributes: AttributeSet<'src>,
    quiet: bool,
  ) -> CompileResult<'src, UnresolvedRecipe<'src>> {
    let name = self.parse_name()?;

    attributes.ensure_valid_attributes(ItemKind::Recipe, *name)?;

    let mut positional = Vec::new();

    let mut longs = HashSet::new();
    let mut shorts = HashSet::new();

    let mut arg_attributes = BTreeMap::new();

    for attribute in &attributes {
      let Attribute::Arg {
        flag,
        help: _,
        help_property: _,
        long,
        long_key,
        max,
        max_key,
        min,
        min_key,
        multiple,
        name: arg,
        pattern: _,
        pattern_property: _,
        short,
        short_key,
        value,
      } = attribute
      else {
        continue;
      };

      if let Some(token) = flag {
        self.list_feature(ListFeature::Flag, *token);
      }

      if let Some(token) = multiple {
        self.list_feature(ListFeature::Multiple, *token);
      }

      if let Some(token) = max_key {
        self.list_feature(ListFeature::ArgMax, **token);
      }

      if let Some(token) = min_key {
        self.list_feature(ListFeature::ArgMin, **token);
      }

      if let Some(option) = long
        && !longs.insert(&option.cooked)
      {
        return Err(long_key.map_or(option.token, |name| *name).error(
          CompileErrorKind::DuplicateOption {
            option: Switch::Long(option.cooked.clone()),
            recipe: name.lexeme(),
          },
        ));
      }

      if let Some(option) = short
        && let Some(short) = option.cooked.chars().next()
        && !shorts.insert(short)
      {
        return Err(short_key.map_or(option.token, |name| *name).error(
          CompileErrorKind::DuplicateOption {
            option: Switch::Short(short),
            recipe: name.lexeme(),
          },
        ));
      }

      arg_attributes.insert(
        arg.cooked.clone(),
        ArgAttribute {
          flag: flag.is_some(),
          name: arg.token,
          long: long.as_ref().map(|long| long.cooked.clone()),
          max: max_key.map(|key| (key, max.unwrap())),
          min: min_key.map(|key| (key, min.unwrap())),
          multiple: multiple.is_some(),
          short: short.as_ref().and_then(|short| short.cooked.chars().next()),
          value: value.clone(),
        },
      );
    }

    while self.next_is(Identifier) || self.next_is(Dollar) {
      positional.push(self.parse_parameter(&mut arg_attributes, ParameterKind::Singular)?);
    }

    let kind = if self.accepted(Plus)? {
      ParameterKind::Plus
    } else if self.accepted(Asterisk)? {
      ParameterKind::Star
    } else {
      ParameterKind::Singular
    };

    let variadic = if kind.is_variadic() {
      let variadic = self.parse_parameter(&mut arg_attributes, kind)?;

      self.forbid(Identifier, |token| {
        token.error(CompileErrorKind::ParameterFollowsVariadicParameter {
          parameter: token.lexeme(),
        })
      })?;

      Some(variadic)
    } else {
      None
    };

    self.expect(Colon)?;

    if let Some((argument, ArgAttribute { name, .. })) = arg_attributes.pop_first() {
      return Err(name.error(CompileErrorKind::UndefinedArgAttribute { argument }));
    }

    let mut dependencies = Vec::new();

    while let Some(dependency) = self.accept_dependency()? {
      dependencies.push(dependency);
    }

    let priors = dependencies.len();

    if self.accepted(AmpersandAmpersand)? {
      let mut subsequents = Vec::new();

      while let Some(subsequent) = self.accept_dependency()? {
        subsequents.push(subsequent);
      }

      if subsequents.is_empty() {
        return Err(self.unexpected_token()?);
      }

      dependencies.append(&mut subsequents);
    }

    if self.next_are(&[Comment, Eol, Indent]) || self.next_are(&[Eol, Indent]) {
      self.expect_eol()?;
    }

    let body = self.parse_body()?;

    let shebang = body.first().is_some_and(Line::is_shebang);

    let script = attributes.contains(AttributeKind::Script);

    if attributes.contains(AttributeKind::WorkingDirectory)
      && attributes.contains(AttributeKind::NoCd)
    {
      return Err(
        name.error(CompileErrorKind::NoCdAndWorkingDirectoryAttribute {
          recipe: name.lexeme(),
        }),
      );
    }

    if attributes.contains(AttributeKind::ExitMessage)
      && attributes.contains(AttributeKind::NoExitMessage)
    {
      return Err(
        name.error(CompileErrorKind::ExitMessageAndNoExitMessageAttribute {
          recipe: name.lexeme(),
        }),
      );
    }

    if attributes.contains(AttributeKind::Script) && attributes.contains(AttributeKind::Shell) {
      return Err(name.error(CompileErrorKind::ScriptAndShellAttribute {
        recipe: name.lexeme(),
      }));
    }

    let private = name.lexeme().starts_with('_') || attributes.private();

    let doc = self.take_doc_comment(&attributes);

    Ok(Recipe {
      attributes,
      body,
      dependencies,
      doc,
      file_depth: self.file_depth,
      import_offsets: self.import_offsets.clone(),
      module_path: None,
      name,
      number: self.numerator.next_recipe(),
      parameters: positional.into_iter().chain(variadic).collect(),
      priors,
      private,
      quiet,
      recipe_path: None,
      shebang: shebang || script,
      variable_references: HashSet::new(),
    })
  }

  /// Parse a recipe parameter
  fn parse_parameter(
    &mut self,
    arg_attributes: &mut BTreeMap<String, ArgAttribute<'src>>,
    kind: ParameterKind,
  ) -> CompileResult<'src, Parameter<'src>> {
    let export = self.accepted(Dollar)?;

    let name = self.parse_name()?;

    let default = if self.accepted(Equals)? {
      Some(self.parse_value()?)
    } else {
      None
    };

    let mut flag = false;
    let help = None;
    let mut long = None;
    let mut max = None;
    let mut min = None;
    let mut multiple = false;
    let pattern = None;
    let mut short = None;
    let mut value = None;

    if let Some(arg) = arg_attributes.remove(name.lexeme()) {
      flag = arg.flag;
      long = arg.long;
      max = arg.max;
      min = arg.min;
      multiple = arg.multiple;
      short = arg.short;
      value = arg.value;
    }

    if flag && default.is_some() {
      return Err(name.error(CompileErrorKind::FlagWithDefault {
        parameter: name.lexeme().into(),
      }));
    }

    if let Some((key, _)) = max.or(min)
      && !multiple
      && !kind.is_variadic()
    {
      return Err(key.error(CompileErrorKind::ArgAttributeRequiresMultipleOrVariadic { key }));
    }

    Ok(Parameter {
      default,
      export,
      flag,
      help,
      kind,
      long,
      max: max.map(|(_key, max)| max),
      min: min.map(|(_key, min)| min),
      multiple,
      name,
      number: self.numerator.next_binding(),
      pattern,
      short,
      value,
    })
  }

  /// Parse the body of a recipe
  fn parse_body(&mut self) -> CompileResult<'src, Vec<Line<'src>>> {
    let mut lines = Vec::new();

    if self.accepted(Indent)? {
      while !self.accepted(Dedent)? {
        let mut fragments = Vec::new();
        let number = self
          .tokens
          .get(self.next_token)
          .map(|token| token.line)
          .unwrap_or_default();

        if !self.accepted(Eol)? {
          while !(self.accepted(Eol)? || self.next_is(Dedent)) {
            if let Some(token) = self.accept(Text)? {
              fragments.push(Fragment::Text { token });
            } else if self.accepted(InterpolationStart)? {
              fragments.push(Fragment::Interpolation {
                expression: self.parse_expression()?,
              });
              self.expect(InterpolationEnd)?;
            } else {
              return Err(self.unexpected_token()?);
            }
          }
        }

        lines.push(Line { fragments, number });
      }
    }

    while lines.last().is_some_and(Line::is_empty) {
      lines.pop();
    }

    Ok(lines)
  }

  /// Parse a boolean setting value
  fn parse_set_bool(&mut self) -> CompileResult<'src, bool> {
    if !self.accepted(ColonEquals)? {
      return Ok(true);
    }

    let identifier = self.expect(Identifier)?;

    let value = if Keyword::True == identifier.lexeme() {
      true
    } else if Keyword::False == identifier.lexeme() {
      false
    } else {
      return Err(identifier.error(CompileErrorKind::ExpectedKeyword {
        expected: vec![Keyword::True, Keyword::False],
        found: identifier,
      }));
    };

    Ok(value)
  }

  /// Parse a setting
  fn parse_set(&mut self, attributes: AttributeSet<'src>) -> CompileResult<'src, Set<'src>> {
    self.presume_keyword(Keyword::Set)?;
    let name = Name::from_identifier(self.presume(Identifier)?);
    let lexeme = name.lexeme();
    let Some(keyword) = Keyword::from_lexeme(lexeme) else {
      return Err(name.error(CompileErrorKind::UnknownSetting {
        setting: name.lexeme(),
      }));
    };

    attributes.ensure_valid_attributes(ItemKind::Setting, *name)?;

    let set_bool = match keyword {
      Keyword::AllowDuplicateRecipes => {
        Some(Setting::AllowDuplicateRecipes(self.parse_set_bool()?))
      }
      Keyword::AllowDuplicateVariables => {
        Some(Setting::AllowDuplicateVariables(self.parse_set_bool()?))
      }
      Keyword::DefaultList => Some(Setting::DefaultList(self.parse_set_bool()?)),
      Keyword::DefaultScript => Some(Setting::DefaultScript(self.parse_set_bool()?)),
      Keyword::DotenvLoad => Some(Setting::DotenvLoad(self.parse_set_bool()?)),
      Keyword::DotenvOverride => Some(Setting::DotenvOverride(self.parse_set_bool()?)),
      Keyword::DotenvRequired => Some(Setting::DotenvRequired(self.parse_set_bool()?)),
      Keyword::Export => Some(Setting::Export(self.parse_set_bool()?)),
      Keyword::Fallback => Some(Setting::Fallback(self.parse_set_bool()?)),
      Keyword::Guards => Some(Setting::Guards(self.parse_set_bool()?)),
      Keyword::IgnoreComments => Some(Setting::IgnoreComments(self.parse_set_bool()?)),
      Keyword::Lazy => Some(Setting::Lazy(self.parse_set_bool()?)),
      Keyword::Lists => {
        self.unstable_features.insert(UnstableFeature::ListsSetting);
        Some(Setting::Lists(self.parse_set_bool()?))
      }
      Keyword::NoCd => Some(Setting::NoCd(self.parse_set_bool()?)),
      Keyword::NoExitMessage => Some(Setting::NoExitMessage(self.parse_set_bool()?)),
      Keyword::PositionalArguments => Some(Setting::PositionalArguments(self.parse_set_bool()?)),
      Keyword::Quiet => Some(Setting::Quiet(self.parse_set_bool()?)),
      Keyword::Unstable => Some(Setting::Unstable(self.parse_set_bool()?)),
      Keyword::WindowsPowershell => Some(Setting::WindowsPowerShell(self.parse_set_bool()?)),
      _ => None,
    };

    if let Some(value) = set_bool {
      return Ok(Set {
        attributes,
        name,
        value,
      });
    }

    self.expect(ColonEquals)?;

    let set_value = match keyword {
      Keyword::DotenvCommand => Some(Setting::DotenvCommand(self.parse_expression()?)),
      Keyword::DotenvFilename => Some(Setting::DotenvFilename(self.parse_expression()?)),
      Keyword::DotenvPath => Some(Setting::DotenvPath(self.parse_expression()?)),
      Keyword::Indentation => {
        let expression = self.parse_expression()?;

        let Expression::StringLiteral { string_literal } = expression else {
          return Err(name.error(CompileErrorKind::SettingExpression { setting: keyword }));
        };

        if string_literal.expand || string_literal.kind.indented || string_literal.part.is_some() {
          return Err(name.error(CompileErrorKind::SettingExpression { setting: keyword }));
        }

        let indentation = string_literal
          .cooked
          .parse::<Indentation>()
          .map_err(|message| {
            string_literal
              .token
              .error(CompileErrorKind::InvalidIndentation { message })
          })?;

        Some(Setting::Indentation(string_literal, indentation))
      }
      Keyword::MinimumVersion => {
        let expression = self.parse_expression()?;

        let Expression::StringLiteral { string_literal } = expression else {
          return Err(name.error(CompileErrorKind::SettingExpression { setting: keyword }));
        };

        if string_literal.expand || string_literal.kind.indented || string_literal.part.is_some() {
          return Err(name.error(CompileErrorKind::SettingExpression { setting: keyword }));
        }

        let minimum = string_literal.cooked.parse::<Version>().map_err(|source| {
          string_literal
            .token
            .error(CompileErrorKind::InvalidMinimumVersion {
              source,
              version: string_literal.cooked.clone(),
            })
        })?;

        let current = Version::current();
        if current < minimum {
          return Err(
            string_literal
              .token
              .error(CompileErrorKind::MinimumVersion { current, minimum }),
          );
        }

        Some(Setting::MinimumVersion(string_literal))
      }
      Keyword::ScriptInterpreter => Some(Setting::ScriptInterpreter(self.parse_interpreter()?)),
      Keyword::Shell => Some(Setting::Shell(self.parse_interpreter()?)),
      Keyword::Tempdir => Some(Setting::Tempdir(self.parse_expression()?)),
      Keyword::WindowsShell => Some(Setting::WindowsShell(self.parse_interpreter()?)),
      Keyword::WorkingDirectory => Some(Setting::WorkingDirectory(self.parse_expression()?)),
      _ => None,
    };

    if let Some(value) = set_value {
      return Ok(Set {
        attributes,
        name,
        value,
      });
    }

    Err(name.error(CompileErrorKind::UnknownSetting {
      setting: name.lexeme(),
    }))
  }

  /// Parse interpreter setting value, i.e., `['sh', '-eu']`
  fn parse_interpreter(&mut self) -> CompileResult<'src, Interpreter<Expression<'src>>> {
    self.expect(BracketL)?;

    let command = self.parse_expression()?;

    let mut arguments = Vec::new();

    if self.accepted(Comma)? {
      while !self.next_is(BracketR) {
        arguments.push(self.parse_expression()?);

        if !self.accepted(Comma)? {
          break;
        }
      }
    }

    self.expect(BracketR)?;

    Ok(Interpreter { command, arguments })
  }

  /// Item attributes, i.e., `[macos]` or `[confirm: "warning!"]`
  fn parse_attributes(&mut self) -> CompileResult<'src, Option<(Token<'src>, AttributeSet<'src>)>> {
    let mut arg_attributes = BTreeMap::new();
    let mut attributes = Vec::new();
    let mut kinds = BTreeMap::new();

    let mut token = None;

    while let Some(bracket) = self.accept(BracketL)? {
      token.get_or_insert(bracket);

      loop {
        let name = self.parse_name()?;

        let kind = name.lexeme().parse::<AttributeKind>().map_err(|_| {
          name.error(CompileErrorKind::UnknownAttribute {
            attribute: name.lexeme(),
          })
        })?;

        if kind == AttributeKind::Cache {
          self
            .unstable_features
            .insert(UnstableFeature::CachedRecipes);
        }

        let mut arguments = Vec::new();
        let mut keyword_arguments = BTreeMap::new();

        if self.accepted(Colon)? {
          let token = self.next()?;
          let expression = self.parse_expression()?;
          arguments.push((token, expression));
        } else if self.accepted(ParenL)? {
          if !self.next_is(ParenR) {
            loop {
              if kind.accepts_keyword_arguments()
                && self.next_is(Identifier)
                && !self.next_is_shell_expanded_string()
              {
                let key = self.parse_name()?;

                let value = self
                  .accepted(Equals)?
                  .then(|| self.parse_expression())
                  .transpose()?;

                if keyword_arguments
                  .insert(key.lexeme(), (key, value))
                  .is_some()
                {
                  return Err(key.error(CompileErrorKind::DuplicateAttributeKey {
                    attribute: name.lexeme(),
                    key: key.lexeme(),
                  }));
                }
              } else {
                let token = self.next()?;
                let expression = self.parse_expression()?;

                if !keyword_arguments.is_empty() {
                  return Err(token.error(CompileErrorKind::AttributePositionalFollowsKeyword));
                }

                arguments.push((token, expression));
              }

              if !self.accepted(Comma)? || self.next_is(ParenR) {
                break;
              }
            }
          }

          self.expect(ParenR)?;
        }

        let attribute = Attribute::new(name, kind, arguments, keyword_arguments)?;

        let first = if attribute.repeatable() {
          None
        } else {
          kinds.get(&attribute.kind())
        };

        if let Some(&first) = first {
          return Err(name.error(CompileErrorKind::DuplicateAttribute {
            attribute: name,
            first,
          }));
        }

        if let Attribute::Arg { name: arg, .. } = &attribute {
          if let Some(&first) = arg_attributes.get(&arg.cooked) {
            return Err(name.error(CompileErrorKind::DuplicateArgAttribute {
              arg: arg.cooked.clone(),
              first,
            }));
          }

          arg_attributes.insert(arg.cooked.clone(), name.line);
        }

        kinds.insert(attribute.kind(), name.line);

        attributes.push((attribute, name));

        if !self.accepted(Comma)? {
          break;
        }
      }
      self.expect(BracketR)?;
      self.expect_eol()?;
    }

    if attributes.is_empty() {
      Ok(None)
    } else {
      Ok(Some((token.unwrap(), attributes.into_iter().collect())))
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, CompileErrorKind::*, pretty_assertions::assert_eq};

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
    let tokens = Lexer::test_lex(&unindented).expect("lexing failed");
    let have = Parser::parse_tokens(&mut Numerator::new(), &tokens)
      .expect("parsing failed")
      .tree();
    if have != want {
      println!("parsed text: {unindented}");
      println!("expected:    {want}");
      println!("but got:     {have}");
      println!("tokens:      {tokens:?}");
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
      found:  $found:expr,
    ) => {
      #[test]
      fn $name() {
        unexpected_token($input, $offset, $line, $column, $width, $found);
      }
    };
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

  #[track_caller]
  fn error(
    src: &str,
    offset: usize,
    line: usize,
    column: usize,
    length: usize,
    kind: CompileErrorKind,
  ) {
    let tokens = Lexer::test_lex(src).expect("Lexing failed in parse test...");

    match Parser::parse_tokens(&mut Numerator::new(), &tokens) {
      Ok(_) => panic!("Parsing unexpectedly succeeded"),
      Err(have) => {
        let want = CompileError {
          token: Token {
            kind: have.token.kind,
            src,
            offset,
            line,
            column,
            length,
            path: "justfile".as_ref(),
          },
          kind: kind.into(),
        };
        assert_eq!(have, want);
      }
    }
  }

  #[track_caller]
  fn unexpected_token(
    src: &str,
    offset: usize,
    line: usize,
    column: usize,
    length: usize,
    found: TokenKind,
  ) {
    let tokens = Lexer::test_lex(src).expect("Lexing failed in parse test...");

    match Parser::parse_tokens(&mut Numerator::new(), &tokens) {
      Ok(_) => panic!("Parsing unexpectedly succeeded"),
      Err(have) => {
        assert_eq!(
          have.token,
          Token {
            kind: have.token.kind,
            src,
            offset,
            line,
            column,
            length,
            path: "justfile".as_ref(),
          },
        );
        match *have.kind {
          UnexpectedToken { found: actual, .. } => assert_eq!(actual, found),
          kind => panic!("expected `UnexpectedToken`, but got: {kind:?}"),
        }
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
    name: alias_with_attribute,
    text: "[private]\nalias t := test",
    tree: (justfile (alias t test)),
  }

  test! {
    name: alias_modulepath,
    text: "alias fbb := foo::bar::baz",
    tree: (justfile (alias fbb (foo bar baz))),
  }

  test! {
    name: single_argument_attribute_shorthand,
    text: "[group: 'foo']\nbar:",
    tree: (justfile (recipe bar)),
  }

  test! {
    name: single_argument_attribute_shorthand_multiple_same_line,
    text: "[group: 'foo', group: 'bar']\nbaz:",
    tree: (justfile (recipe baz)),
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
    text: "alias t := test",
    tree: (justfile
      (alias t test)
    ),
  }

  test! {
      name: recipe_named_alias,
      text: "
      [private]
      alias:
        echo 'echoing alias'
          ",
    tree: (justfile
      (recipe alias (body ("echo 'echoing alias'")))
    ),
  }

  test! {
    name: export,
    text: "export x := 'hello'",
    tree: (justfile (assignment #export x "hello")),
  }

  test! {
    name: private_export,
    text: "
      [private]
      export x := 'hello'
    ",
    tree: (justfile (assignment #export x "hello")),
  }

  test! {
    name: export_equals,
    text: "export x := 'hello'",
    tree: (justfile
      (assignment #export x "hello")
    ),
  }

  test! {
    name: assignment,
    text: "x := 'hello'",
    tree: (justfile (assignment x "hello")),
  }

  test! {
    name: private_assignment,
    text: "
      [private]
      x := 'hello'
      ",
    tree: (justfile (assignment x "hello")),
  }

  test! {
    name: assignment_equals,
    text: "x := 'hello'",
    tree: (justfile
      (assignment x "hello")
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
    name: list_concatenation_single,
    text: "x := a ++ b",
    tree: (justfile (assignment x (++ a b))),
  }

  test! {
    name: addition_chained,
    text: "x := a + b + c",
    tree: (justfile (assignment x (+ a (+ b c)))),
  }

  test! {
    name: call_one_arg,
    text: "x := env_var(y)",
    tree: (justfile (assignment x (call env_var y))),
  }

  test! {
    name: call_multiple_args,
    text: "x := env_var_or_default(y, z)",
    tree: (justfile (assignment x (call env_var_or_default y z))),
  }

  test! {
    name: call_trailing_comma,
    text: "x := env_var(y,)",
    tree: (justfile (assignment x (call env_var y))),
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
    text: "foo bar='baz':",
    tree: (justfile (recipe foo (params (bar "baz")))),
  }

  test! {
    name: recipe_default_multiple,
    text: "foo bar='baz' bob='biz':",
    tree: (justfile (recipe foo (params (bar "baz") (bob "biz")))),
  }

  test! {
    name: recipe_plus_variadic,
    text: "foo +bar:",
    tree: (justfile (recipe foo (params +(bar)))),
  }

  test! {
    name: recipe_star_variadic,
    text: "foo *bar:",
    tree: (justfile (recipe foo (params *(bar)))),
  }

  test! {
    name: recipe_variadic_string_default,
    text: "foo +bar='baz':",
    tree: (justfile (recipe foo (params +(bar "baz")))),
  }

  test! {
    name: recipe_variadic_variable_default,
    text: "foo +bar=baz:",
    tree: (justfile (recipe foo (params +(bar baz)))),
  }

  test! {
    name: recipe_variadic_addition_group_default,
    text: "foo +bar=(baz + bob):",
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
    name: recipe_dependency_parenthesis,
    text: "foo: (bar)",
    tree: (justfile (recipe foo (deps bar))),
  }

  test! {
    name: recipe_dependency_argument_string,
    text: "foo: (bar 'baz')",
    tree: (justfile (recipe foo (deps (bar "baz")))),
  }

  test! {
    name: recipe_dependency_argument_identifier,
    text: "foo: (bar baz)",
    tree: (justfile (recipe foo (deps (bar baz)))),
  }

  test! {
    name: recipe_dependency_argument_concatenation,
    text: "foo: (bar 'a' + 'b' 'c' + 'd')",
    tree: (justfile (recipe foo (deps (bar (+ 'a' 'b') (+ 'c' 'd'))))),
  }

  test! {
    name: recipe_subsequent,
    text: "foo: && bar",
    tree: (justfile (recipe foo (sups bar))),
  }

  test! {
    name: recipe_line_single,
    text: "foo:\n bar",
    tree: (justfile (recipe foo (body ("bar")))),
  }

  test! {
    name: recipe_dependency_module,
    text: "foo: bar::baz",
    tree: (justfile (recipe foo (deps (bar baz)))),
  }

  test! {
    name: recipe_dependency_parenthesis_module,
    text: "foo: (bar::baz)",
    tree: (justfile (recipe foo (deps (bar baz)))),
  }

  test! {
    name: recipe_dependency_module_mixed,
      text: "foo: bar::baz qux",
    tree: (justfile (recipe foo (deps (bar baz) qux))),
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
    tree: (justfile (comment "# foo")),
  }

  test! {
    name: comment_before_alias,
    text: "# foo\nalias x := y",
    tree: (justfile (comment "# foo") (alias x y)),
  }

  test! {
    name: comment_after_alias,
    text: "alias x := y # foo",
    tree: (justfile (alias x y) (comment "# foo")),
  }

  test! {
    name: comment_assignment,
    text: "x := y # foo",
    tree: (justfile (assignment x y) (comment "# foo")),
  }

  test! {
    name: comment_export,
    text: "export x := y # foo",
    tree: (justfile (assignment #export x y) (comment "# foo")),
  }

  test! {
    name: comment_recipe,
    text: "foo: # bar",
    tree: (justfile (recipe foo) (comment "# bar")),
  }

  test! {
    name: comment_recipe_dependencies,
    text: "foo: bar # baz",
    tree: (justfile (recipe foo (deps bar)) (comment "# baz")),
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
    tree: (justfile (comment "# foo") (assignment x y) (recipe bar)),
  }

  test! {
    name: doc_comment_empty_line_clear,
    text: "
      # foo

      bar:
    ",
    tree: (justfile (comment "# foo") (recipe bar)),
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
    name: string_escape_suppress_newline,
    text: r#"
      x := "foo\
      bar"
    "#,
    tree: (justfile (assignment x "foobar")),
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
    name: indented_string_raw_with_dedent,
    text: "
      x := '''
        foo\\t
        bar\\n
      '''
    ",
    tree: (justfile (assignment x "foo\\t\nbar\\n\n")),
  }

  test! {
    name: indented_string_raw_no_dedent,
    text: "
      x := '''
      foo\\t
        bar\\n
      '''
    ",
    tree: (justfile (assignment x "foo\\t\n  bar\\n\n")),
  }

  test! {
    name: indented_string_cooked,
    text: r#"
      x := """
        \tfoo\t
        \tbar\n
      """
    "#,
    tree: (justfile (assignment x "\tfoo\t\n\tbar\n\n")),
  }

  test! {
    name: indented_string_cooked_no_dedent,
    text: r#"
      x := """
      \tfoo\t
        \tbar\n
      """
    "#,
    tree: (justfile (assignment x "\tfoo\t\n  \tbar\n\n")),
  }

  test! {
    name: indented_backtick,
    text: r"
      x := ```
        \tfoo\t
        \tbar\n
      ```
    ",
    tree: (justfile (assignment x (backtick "\\tfoo\\t\n\\tbar\\n\n"))),
  }

  test! {
    name: indented_backtick_no_dedent,
    text: r"
      x := ```
      \tfoo\t
        \tbar\n
      ```
    ",
    tree: (justfile (assignment x (backtick "\\tfoo\\t\n  \\tbar\\n\n"))),
  }

  test! {
    name: recipe_variadic_with_default_after_default,
    text: "
      f a=b +c=d:
    ",
    tree: (justfile (recipe f (params (a b) +(c d)))),
  }

  test! {
    name: parameter_default_concatenation_variable,
    text: "
      x := '10'

      f y=(`echo hello` + x) +z='foo':
    ",
    tree: (justfile
      (assignment x "10")
      (recipe f (params (y ((+ (backtick "echo hello") x))) +(z "foo")))
    ),
  }

  test! {
    name: parameter_default_multiple,
    text: "
      x := '10'
      f y=(`echo hello` + x) +z=('foo' + 'bar'):
    ",
    tree: (justfile
      (assignment x "10")
      (recipe f (params (y ((+ (backtick "echo hello") x))) +(z ((+ "foo" "bar")))))
    ),
  }

  test! {
    name: parse_raw_string_default,
    text: r"

      foo a='b\t':


    ",
    tree: (justfile (recipe foo (params (a "b\\t")))),
  }

  test! {
    name: parse_alias_after_target,
    text: "
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
      alias f := foo # comment
      foo:
        echo a
    ",
    tree: (justfile
      (alias f foo)
      (comment "# comment")
      (recipe foo (body ("echo a")))
    ),
  }

  test! {
    name: parse_assignment_with_comment,
    text: "
      f := foo # comment
      foo:
        echo a
    ",
    tree: (justfile
      (assignment f foo)
      (comment "# comment")
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
    text: "
      a := '0'
      c := a + b + a + b
      b := '1'
    ",
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
    text: "
      a:
        echo {{  `echo hello` + 'blarg'   }} {{   `echo bob`   }}
    ",
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
    text: "
      x := env_var('foo',)

      a:
        {{env_var_or_default('foo' + 'bar', 'baz',)}} {{env_var(env_var('baz'))}}
    ",
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
    text: "
      f x='abc':
    ",
    tree: (justfile (recipe f (params (x "abc")))),
  }

  test! {
    name: parameter_default_raw_string,
    text: "
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
    name: parameter_default_concatenation_string,
    text: "
      f x=(`echo hello` + 'foo'):
    ",
    tree: (justfile (recipe f (params (x ((+ (backtick "echo hello") "foo")))))),
  }

  test! {
    name: concatenation_in_group,
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
    name: set_export_implicit,
    text: "set export",
    tree: (justfile (set export true)),
  }

  test! {
    name: set_export_true,
    text: "set export := true",
    tree: (justfile (set export true)),
  }

  test! {
    name: set_export_false,
    text: "set export := false",
    tree: (justfile (set export false)),
  }

  test! {
    name: set_dotenv_load_implicit,
    text: "set dotenv-load",
    tree: (justfile (set dotenv_load true)),
  }

  test! {
    name: set_allow_duplicate_recipes_implicit,
    text: "set allow-duplicate-recipes",
    tree: (justfile (set allow_duplicate_recipes true)),
  }

  test! {
    name: set_allow_duplicate_variables_implicit,
    text: "set allow-duplicate-variables",
    tree: (justfile (set allow_duplicate_variables true)),
  }

  test! {
    name: set_dotenv_load_true,
    text: "set dotenv-load := true",
    tree: (justfile (set dotenv_load true)),
  }

  test! {
    name: set_dotenv_load_false,
    text: "set dotenv-load := false",
    tree: (justfile (set dotenv_load false)),
  }

  test! {
    name: set_positional_arguments_implicit,
    text: "set positional-arguments",
    tree: (justfile (set positional_arguments true)),
  }

  test! {
    name: set_positional_arguments_true,
    text: "set positional-arguments := true",
    tree: (justfile (set positional_arguments true)),
  }

  test! {
    name: set_quiet_implicit,
    text: "set quiet",
    tree: (justfile (set quiet true)),
  }

  test! {
    name: set_quiet_true,
    text: "set quiet := true",
    tree: (justfile (set quiet true)),
  }

  test! {
    name: set_quiet_false,
    text: "set quiet := false",
    tree: (justfile (set quiet false)),
  }

  test! {
    name: set_no_cd,
    text: "set no-cd := true",
    tree: (justfile (set no_cd true)),
  }

  test! {
    name: set_positional_arguments_false,
    text: "set positional-arguments := false",
    tree: (justfile (set positional_arguments false)),
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

  test! {
    name: set_windows_powershell_implicit,
    text: "set windows-powershell",
    tree: (justfile (set windows_powershell true)),
  }

  test! {
    name: set_windows_powershell_true,
    text: "set windows-powershell := true",
    tree: (justfile (set windows_powershell true)),
  }

  test! {
    name: set_windows_powershell_false,
    text: "set windows-powershell := false",
    tree: (justfile (set windows_powershell false)),
  }

  test! {
    name: set_working_directory,
    text: "set working-directory := 'foo'",
    tree: (justfile (set working_directory "foo")),
  }

  test! {
    name: conditional,
    text: "a := if b { c } else { d }",
    tree: (justfile (assignment a (if b c d))),
  }

  test! {
    name: conditional_without_otherwise,
    text: "a := if b { c }",
    tree: (justfile (assignment a (if b c))),
  }

  test! {
    name: conditional_inverted,
    text: "a := if b != c { d } else { e }",
    tree: (justfile (assignment a (if (!= b c) d e))),
  }

  test! {
    name: conditional_concatenations,
    text: "a := if b0 + b1 { c0 + c1 } else { d0 + d1 }",
    tree: (justfile (assignment a (if (+ b0 b1) (+ c0 c1) (+ d0 d1)))),
  }

  test! {
    name: conditional_nested_lhs,
    text: "a := if if b == c { d } else { e } == c { d } else { e }",
    tree: (justfile (assignment a (if (== (if (== b c) d e) c) d e))),
  }

  test! {
    name: conditional_nested_rhs,
    text: "a := if c == if b == c { d } else { e } { d } else { e }",
    tree: (justfile (assignment a (if (== c (if (== b c) d e)) d e))),
  }

  test! {
    name: conditional_nested_then,
    text: "a := if b { if c { d } else { e } } else { f }",
    tree: (justfile (assignment a (if b (if c d e) f))),
  }

  test! {
    name: conditional_nested_otherwise,
    text: "a := if b { c } else { if d { e } else { f } }",
    tree: (justfile (assignment a (if b c (if d e f)))),
  }

  test! {
    name: comparison,
    text: "a := b == c",
    tree: (justfile (assignment a (== b c))),
  }

  test! {
    name: comparison_binds_looser_than_concatenation,
    text: "a := b + c == d + e",
    tree: (justfile (assignment a (== (+ b c) (+ d e)))),
  }

  test! {
    name: comparison_binds_tighter_than_logical_operators,
    text: "a := b == c && d == e || f == g",
    tree: (justfile (assignment a (|| (&& (== b c) (== d e)) (== f g)))),
  }

  test! {
    name: negation,
    text: "a := !b",
    tree: (justfile (assignment a (! b))),
  }

  test! {
    name: negation_binds_tighter_than_comparison,
    text: "a := !b == c",
    tree: (justfile (assignment a (== (! b) c))),
  }

  test! {
    name: double_negation,
    text: "a := !!b",
    tree: (justfile (assignment a (! (! b)))),
  }

  test! {
    name: import,
    text: "import \"some/file/path.txt\"     \n",
    tree: (justfile (import "some/file/path.txt")),
  }

  test! {
    name: optional_import,
    text: "import? \"some/file/path.txt\"     \n",
    tree: (justfile (import ? "some/file/path.txt")),
  }

  test! {
    name: module_with,
    text: "mod foo",
    tree: (justfile (mod foo )),
  }

  test! {
    name: optional_module,
    text: "mod? foo",
    tree: (justfile (mod ? foo)),
  }

  test! {
    name: module_with_path,
    text: "mod foo \"some/file/path.txt\"     \n",
    tree: (justfile (mod foo "some/file/path.txt")),
  }

  test! {
    name: optional_module_with_path,
    text: "mod? foo \"some/file/path.txt\"     \n",
    tree: (justfile (mod ? foo "some/file/path.txt")),
  }

  test! {
    name: assert,
    text: "a := assert(b, \"error\")",
    tree: (justfile (assignment a (assert b "error"))),
  }

  test! {
    name: assert_conditional_condition,
    text: "foo := assert(if a { b } else { c }, \"error\")",
    tree: (justfile (assignment foo (assert (if a b c) "error"))),
  }

  test! {
    name: format_string_simple,
    text: "foo := f'abc'",
    tree: (justfile (assignment foo (format "abc"))),
  }

  test! {
    name: format_string_expression,
    text: "foo := f'foo{{ 'abc' + 'xyz' }}bar'",
    tree: (justfile (assignment foo (format "foo" (+ "abc" "xyz") "bar"))),
  }

  test! {
    name: format_string_complex,
    text: "foo := f'foo{{ 'abc' + 'xyz' }}bar{{ 'hello' }}goodbye'",
    tree: (justfile (assignment foo (format "foo" (+ "abc" "xyz") "bar" "hello" "goodbye"))),
  }

  error! {
    name:   alias_syntax_multiple_rhs,
    input:  "alias foo := bar baz",
    offset: 17,
    line:   0,
    column: 17,
    width:  3,
    found:  Identifier,
  }

  error! {
    name:   alias_syntax_no_rhs,
    input:  "alias foo := \n",
    offset: 13,
    line:   0,
    column: 13,
    width:  1,
    found:  Eol,
  }

  error! {
    name:   alias_syntax_colon_end,
    input:  "alias foo := bar::\n",
    offset: 18,
    line:   0,
    column: 18,
    width:  1,
    found:  Eol,
  }

  error! {
    name:   alias_syntax_single_colon,
    input:  "alias foo := bar:baz",
    offset: 16,
    line:   0,
    column: 16,
    width:  1,
    found:  Colon,
  }

  error! {
    name:   missing_colon,
    input:  "a b c\nd e f",
    offset:  5,
    line:   0,
    column: 5,
    width:  1,
    found:  Eol,
  }

  error! {
    name:   missing_default_eol,
    input:  "hello arg=\n",
    offset:  10,
    line:   0,
    column: 10,
    width:  1,
    kind:   UnexpectedToken {
      expected: vec![
        Backtick,
        Bang,
        BracketL,
        Identifier,
        ParenL,
        StringToken,
      ],
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
    found:  Eof,
  }

  error! {
    name:   missing_eol,
    input:  "a b c: z =",
    offset:  9,
    line:    0,
    column:  9,
    width:   1,
    found:  Equals,
  }

  error! {
    name:   unexpected_brace,
    input:  "{{",
    offset:  0,
    line:   0,
    column: 0,
    width:  1,
    found:  BraceL,
  }

  error! {
    name:   unclosed_parenthesis_in_expression,
    input:  "x := foo(",
    offset: 9,
    line:   0,
    column: 9,
    width:  0,
    found:  Eof,
  }

  error! {
    name:   plus_following_parameter,
    input:  "a b c+:",
    offset: 6,
    line:   0,
    column: 6,
    width:  1,
    found:  Colon,
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
    found:  Eof,
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
    name:   concatenation_in_default,
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
    found:  BracketR,
  }

  error! {
    name:   set_shell_bad_comma,
    input:  "set shell := ['bash',",
    offset: 21,
    line:   0,
    column: 21,
    width:  0,
    found:  Eof,
  }

  error! {
    name:   set_shell_bad,
    input:  "set shell := ['bash'",
    offset: 20,
    line:   0,
    column: 20,
    width:  0,
    found:  Eof,
  }

  error! {
    name:   empty_attribute,
    input:  "[]\nsome_recipe:\n @exit 3",
    offset: 1,
    line:   0,
    column: 1,
    width:  1,
    found:  BracketR,
  }

  error! {
    name:   unknown_attribute,
    input:  "[unknown]\nsome_recipe:\n @exit 3",
    offset: 1,
    line:   0,
    column: 1,
    width:  7,
    kind:   UnknownAttribute { attribute: "unknown" },
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
}
