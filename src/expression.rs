use crate::common::*;

/// An expression. Note that the Just language grammar has both an `expression`
/// production of additions (`a + b`) and values, and a `value` production of
/// all other value types (for example strings, function calls, and
/// parenthetical groups).
///
/// The parser parses both values and expressions into `Expression`s.
#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Expression<'src> {
  /// `contents`
  Backtick {
    contents: String,
    token: Token<'src>,
  },
  /// `name(arguments)`
  Call { thunk: Thunk<'src> },
  /// `lhs + rhs`
  Concatination {
    lhs: Box<Expression<'src>>,
    rhs: Box<Expression<'src>>,
  },
  /// `if lhs == rhs { then } else { otherwise }`
  Conditional {
    lhs: Box<Expression<'src>>,
    rhs: Box<Expression<'src>>,
    then: Box<Expression<'src>>,
    otherwise: Box<Expression<'src>>,
    operator: ConditionalOperator,
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
      Expression::Backtick { token, .. } => write!(f, "{}", token.lexeme()),
      Expression::Concatination { lhs, rhs } => write!(f, "{} + {}", lhs, rhs),
      Expression::Conditional {
        lhs,
        rhs,
        then,
        otherwise,
        operator,
      } => write!(
        f,
        "if {} {} {} {{ {} }} else {{ {} }}",
        lhs, operator, rhs, then, otherwise
      ),
      Expression::StringLiteral { string_literal } => write!(f, "{}", string_literal),
      Expression::Variable { name } => write!(f, "{}", name.lexeme()),
      Expression::Call { thunk } => write!(f, "{}", thunk),
      Expression::Group { contents } => write!(f, "({})", contents),
    }
  }
}

impl<'src> Serialize for Expression<'src> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    todo!();
  }
}
