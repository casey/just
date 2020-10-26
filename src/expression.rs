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
  /// `if lhs == rhs { then } else { otherwise }`
  /// TODO:
  /// - Document:
  ///   - short circuits
  ///   - can use `==` or `!=`
  /// - Test:
  ///   - new token lexing
  ///   - parsing
  ///   - test that expressions are resolved in all components of `if`
  ///   - test all new error messages
  ///   - test that backticks are evaluated in lhs and rhs
  /// - Add `!=`
  /// - Consider how to do line continuation
  /// - can i do line continuation if there's an open {? what about a `\`?
  /// - test inside of recipe interpolation
  /// - test if inside of lhs, rhs, then, otherwise
  /// - test that unexpected token for Op expects == and !=
  Conditional {
    lhs:       Box<Expression<'src>>,
    rhs:       Box<Expression<'src>>,
    then:      Box<Expression<'src>>,
    otherwise: Box<Expression<'src>>,
    inverted:  bool,
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
      // TODO: This needs to be tested
      Expression::Conditional {
        lhs,
        rhs,
        then,
        otherwise,
        inverted,
      } => write!(
        f,
        "if {} {} {} {{ {} }} else {{ {} }} ",
        lhs,
        if *inverted { "!=" } else { "==" },
        rhs,
        then,
        otherwise
      ),
      Expression::StringLiteral { string_literal } => write!(f, "{}", string_literal),
      Expression::Variable { name } => write!(f, "{}", name.lexeme()),
      Expression::Call { thunk } => write!(f, "{}", thunk),
      Expression::Group { contents } => write!(f, "({})", contents),
    }
  }
}
