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
    contents: String,
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
  Conditional {
    lhs:       Box<Expression<'src>>,
    rhs:       Box<Expression<'src>>,
    then:      Box<Expression<'src>>,
    otherwise: Box<Expression<'src>>,
    inverted:  bool,
  },
  /// f'…', f"…", f'''…''', or f"""…"""
  FormatString {
    kind: StringKind,
    fragments: Vec<StringFragment<'src>>,
  },
  /// f`...` or f```...```
  FormatBacktick {
    start: Token<'src>,
    kind: StringKind,
    fragments: Vec<StringFragment<'src>>,
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
      Expression::FormatString { kind, fragments }
      | Expression::FormatBacktick {
        kind, fragments, ..
      } => {
        write!(f, "f{}", kind.delimiter())?;
        for fragment in fragments {
          write!(f, "{}", fragment)?;
        }
        write!(f, "{}", kind.delimiter())
      }
      Expression::StringLiteral { string_literal } => write!(f, "{}", string_literal),
      Expression::Variable { name } => write!(f, "{}", name.lexeme()),
      Expression::Call { thunk } => write!(f, "{}", thunk),
      Expression::Group { contents } => write!(f, "({})", contents),
    }
  }
}
