use crate::common::*;

/// An expression. Note that the Just language grammar has both an `expression`
/// production of additions (`a + b`) and values, and a `value` production of
/// all other value types (for example strings, function calls, and
/// parenthetical groups).
///
/// The parser parses both values and expressions into `Expression`s.
#[derive(PartialEq, Debug)]
pub(crate) enum Expression<'src> {
  /// `contents`
  Backtick {
    contents: &'src str,
    token:    Token<'src>,
  },
  /// `name(arguments)`
  Call { thunk: Thunk<'src> },
  /// `lhs + rhs`
  Concatination {
    lhs: Box<Expression<'src>>,
    rhs: Box<Expression<'src>>,
  },
  /// `(contents)`
  Group { contents: Box<Expression<'src>> },
  /// `"string_literal"` or `'string_literal'`
  StringLiteral { string_literal: StringLiteral<'src> },
  /// `variable`
  Variable { name: Name<'src> },
}

impl<'src> Expression<'src> {
  pub(crate) fn variables<'expression>(&'expression self) -> Variables<'expression, 'src> {
    Variables::new(self)
  }
}

impl<'src> Display for Expression<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    match self {
      Expression::Backtick { contents, .. } => write!(f, "`{}`", contents),
      Expression::Concatination { lhs, rhs } => write!(f, "{} + {}", lhs, rhs),
      Expression::StringLiteral { string_literal } => write!(f, "{}", string_literal),
      Expression::Variable { name } => write!(f, "{}", name.lexeme()),
      Expression::Call { thunk } => write!(f, "{}", thunk),
      Expression::Group { contents } => write!(f, "({})", contents),
    }
  }
}
