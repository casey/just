use super::*;

/// An expression. Note that the Just language grammar has both an `expression`
/// production of additions (`a + b`) and values, and a `value` production of
/// all other value types (for example strings, function calls, and
/// parenthetical groups).
///
/// The parser parses both values and expressions into `Expression`s.
#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Expression<'src> {
  /// `assert(condition, error)`
  Assert {
    condition: Condition<'src>,
    error: Box<Expression<'src>>,
  },
  /// `contents`
  Backtick {
    contents: String,
    token: Token<'src>,
  },
  /// `name(arguments)`
  Call { thunk: Thunk<'src> },
  /// `lhs + rhs`
  Concatenation {
    lhs: Box<Expression<'src>>,
    rhs: Box<Expression<'src>>,
  },
  /// `if condition { then } else { otherwise }`
  Conditional {
    condition: Condition<'src>,
    then: Box<Expression<'src>>,
    otherwise: Box<Expression<'src>>,
  },
  /// `(contents)`
  Group { contents: Box<Expression<'src>> },
  /// `lhs / rhs`
  Join {
    lhs: Option<Box<Expression<'src>>>,
    rhs: Box<Expression<'src>>,
  },
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
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Assert { condition, error } => write!(f, "assert({condition}, {error})"),
      Self::Backtick { token, .. } => write!(f, "{}", token.lexeme()),
      Self::Join { lhs: None, rhs } => write!(f, "/ {rhs}"),
      Self::Join {
        lhs: Some(lhs),
        rhs,
      } => write!(f, "{lhs} / {rhs}"),
      Self::Concatenation { lhs, rhs } => write!(f, "{lhs} + {rhs}"),
      Self::Conditional {
        condition,
        then,
        otherwise,
      } => write!(f, "if {condition} {{ {then} }} else {{ {otherwise} }}"),
      Self::StringLiteral { string_literal } => write!(f, "{string_literal}"),
      Self::Variable { name } => write!(f, "{}", name.lexeme()),
      Self::Call { thunk } => write!(f, "{thunk}"),
      Self::Group { contents } => write!(f, "({contents})"),
    }
  }
}

impl<'src> Serialize for Expression<'src> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      Self::Assert { condition, error } => {
        let mut seq: <S as Serializer>::SerializeSeq = serializer.serialize_seq(None)?;
        seq.serialize_element("assert")?;
        seq.serialize_element(condition)?;
        seq.serialize_element(error)?;
        seq.end()
      }
      Self::Backtick { contents, .. } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("evaluate")?;
        seq.serialize_element(contents)?;
        seq.end()
      }
      Self::Call { thunk } => thunk.serialize(serializer),
      Self::Concatenation { lhs, rhs } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("concatenate")?;
        seq.serialize_element(lhs)?;
        seq.serialize_element(rhs)?;
        seq.end()
      }
      Self::Join { lhs, rhs } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("join")?;
        seq.serialize_element(lhs)?;
        seq.serialize_element(rhs)?;
        seq.end()
      }
      Self::Conditional {
        condition,
        then,
        otherwise,
      } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("if")?;
        seq.serialize_element(condition)?;
        seq.serialize_element(then)?;
        seq.serialize_element(otherwise)?;
        seq.end()
      }
      Self::Group { contents } => contents.serialize(serializer),
      Self::StringLiteral { string_literal } => string_literal.serialize(serializer),
      Self::Variable { name } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("variable")?;
        seq.serialize_element(name)?;
        seq.end()
      }
    }
  }
}
