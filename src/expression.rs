use super::*;

/// An expression. Note that the Just language grammar has both an `expression`
/// production of additions (`a + b`) and values, and a `value` production of
/// all other value types (for example strings, function calls, and
/// parenthetical groups).
///
/// The parser parses both values and expressions into `Expression`s.
#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Expression<'src> {
  /// `lhs && rhs`
  And { lhs: Box<Self>, rhs: Box<Self> },
  /// `assert(condition, error)`
  Assert {
    name: Name<'src>,
    condition: Condition<'src>,
    error: Box<Self>,
  },
  /// `contents`
  Backtick {
    contents: String,
    token: Token<'src>,
  },
  /// `name(arguments)`
  Call { thunk: Thunk<'src> },
  /// `lhs + rhs`
  Concatenation { lhs: Box<Self>, rhs: Box<Self> },
  /// `if condition { then } else { otherwise }`
  Conditional {
    condition: Condition<'src>,
    then: Box<Self>,
    otherwise: Box<Self>,
  },
  // `f"format string"`
  FormatString {
    start: StringLiteral<'src>,
    expressions: Vec<(Self, StringLiteral<'src>)>,
  },
  /// `(contents)`
  Group { contents: Box<Self> },
  /// `lhs / rhs`
  Join {
    lhs: Option<Box<Self>>,
    rhs: Box<Self>,
  },
  /// `lhs || rhs`
  Or { lhs: Box<Self>, rhs: Box<Self> },
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

impl Display for Expression<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::And { lhs, rhs } => write!(f, "{lhs} && {rhs}"),
      Self::Assert {
        condition, error, ..
      } => write!(f, "assert({condition}, {error})"),
      Self::Backtick { token, .. } => write!(f, "{}", token.lexeme()),
      Self::Call { thunk } => write!(f, "{thunk}"),
      Self::Concatenation { lhs, rhs } => write!(f, "{lhs} + {rhs}"),
      Self::Conditional {
        condition,
        then,
        otherwise,
      } => {
        if let Self::Conditional { .. } = **otherwise {
          write!(f, "if {condition} {{ {then} }} else {otherwise}")
        } else {
          write!(f, "if {condition} {{ {then} }} else {{ {otherwise} }}")
        }
      }
      Self::FormatString { start, expressions } => {
        write!(f, "{start}")?;

        for (expression, string) in expressions {
          write!(f, "{expression}{string}")?;
        }

        Ok(())
      }
      Self::Group { contents } => write!(f, "({contents})"),
      Self::Join { lhs: None, rhs } => write!(f, "/ {rhs}"),
      Self::Join {
        lhs: Some(lhs),
        rhs,
      } => write!(f, "{lhs} / {rhs}"),
      Self::Or { lhs, rhs } => write!(f, "{lhs} || {rhs}"),
      Self::StringLiteral { string_literal } => write!(f, "{string_literal}"),
      Self::Variable { name } => write!(f, "{}", name.lexeme()),
    }
  }
}

impl Serialize for Expression<'_> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      Self::And { lhs, rhs } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("and")?;
        seq.serialize_element(lhs)?;
        seq.serialize_element(rhs)?;
        seq.end()
      }
      Self::Assert {
        condition, error, ..
      } => {
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
      Self::FormatString { start, expressions } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("format")?;
        seq.serialize_element(start)?;
        for (expression, string) in expressions {
          seq.serialize_element(expression)?;
          seq.serialize_element(string)?;
        }
        seq.end()
      }
      Self::Group { contents } => contents.serialize(serializer),
      Self::Join { lhs, rhs } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("join")?;
        seq.serialize_element(lhs)?;
        seq.serialize_element(rhs)?;
        seq.end()
      }
      Self::Or { lhs, rhs } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("or")?;
        seq.serialize_element(lhs)?;
        seq.serialize_element(rhs)?;
        seq.end()
      }
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
