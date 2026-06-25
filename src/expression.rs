use super::*;

/// An expression. Note that the Just language grammar has both an `expression`
/// production of additions (`a + b`) and values, and a `value` production of
/// all other value types (for example strings, function calls, and
/// parenthetical groups).
///
/// The parser parses both values and expressions into `Expression`s.
#[derive(PartialEq, Eq, Debug, Clone, Ord, PartialOrd)]
pub(crate) enum Expression<'src> {
  /// `lhs && rhs`
  And { lhs: Box<Self>, rhs: Box<Self> },
  /// `assert(condition, error)`
  Assert {
    condition: Box<Self>,
    message: Option<Box<Self>>,
    name: Name<'src>,
  },
  /// `contents`
  Backtick {
    contents: String,
    token: Token<'src>,
  },
  /// `name(arguments)`
  Call {
    name: Name<'src>,
    arguments: Vec<Expression<'src>>,
  },
  /// `lhs == rhs`
  Comparison {
    lhs: Box<Self>,
    operator: ConditionalOperator,
    operator_token: Token<'src>,
    rhs: Box<Self>,
  },
  /// `lhs + rhs`
  Concatenation {
    lhs: Box<Self>,
    operator: Token<'src>,
    rhs: Box<Self>,
  },
  /// `if condition { then } else { otherwise }`
  Conditional {
    condition: Box<Self>,
    then: Box<Self>,
    otherwise: Option<Box<Self>>,
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
    operator: Token<'src>,
    rhs: Box<Self>,
  },
  /// `[a, b, c]`
  List {
    elements: Vec<Expression<'src>>,
    open: Token<'src>,
  },
  /// `lhs ++ rhs`
  ListConcatenation {
    lhs: Box<Self>,
    operator: Token<'src>,
    rhs: Box<Self>,
  },
  /// `!operand`
  Not { operand: Box<Self> },
  /// `lhs || rhs`
  Or { lhs: Box<Self>, rhs: Box<Self> },
  /// `"string_literal"` or `'string_literal'`
  StringLiteral { string_literal: StringLiteral<'src> },
  /// `variable`
  Variable { name: Name<'src> },
}

impl<'src> Expression<'src> {
  pub(crate) fn references<'a>(&'a self) -> References<'a, 'src> {
    References::new(self)
  }
}

impl Display for Expression<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::And { lhs, rhs } => write!(f, "{lhs} && {rhs}"),
      Self::Assert {
        condition, message, ..
      } => {
        if let Some(error) = message {
          write!(f, "assert({condition}, {error})")
        } else {
          write!(f, "assert({condition})")
        }
      }
      Self::Backtick { token, .. } => write!(f, "{}", token.lexeme()),
      Self::Call { name, arguments } => {
        write!(f, "{name}(")?;
        for (i, argument) in arguments.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "{argument}")?;
        }
        write!(f, ")")
      }
      Self::Comparison {
        lhs, operator, rhs, ..
      } => write!(f, "{lhs} {operator} {rhs}"),
      Self::Concatenation { lhs, rhs, .. } => write!(f, "{lhs} + {rhs}"),
      Self::ListConcatenation { lhs, rhs, .. } => write!(f, "{lhs} ++ {rhs}"),
      Self::Conditional {
        condition,
        then,
        otherwise,
      } => {
        write!(f, "if {condition} {{ {then} }}")?;
        if let Some(otherwise) = otherwise {
          if let Self::Conditional { .. } = **otherwise {
            write!(f, " else {otherwise}")?;
          } else {
            write!(f, " else {{ {otherwise} }}")?;
          }
        }
        Ok(())
      }
      Self::FormatString { start, expressions } => {
        write!(f, "{start}")?;

        for (expression, string) in expressions {
          write!(f, "{expression}{string}")?;
        }

        Ok(())
      }
      Self::Group { contents } => write!(f, "({contents})"),
      Self::Join { lhs: None, rhs, .. } => write!(f, "/ {rhs}"),
      Self::Join {
        lhs: Some(lhs),
        rhs,
        ..
      } => write!(f, "{lhs} / {rhs}"),
      Self::List { elements, .. } => {
        write!(f, "[")?;
        for (i, element) in elements.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "{element}")?;
        }
        write!(f, "]")
      }
      Self::Not { operand } => write!(f, "!{operand}"),
      Self::Or { lhs, rhs } => write!(f, "{lhs} || {rhs}"),
      Self::StringLiteral { string_literal } => write!(f, "{string_literal}"),
      Self::Variable { name } => write!(f, "{name}"),
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
        condition, message, ..
      } => {
        let mut seq: <S as Serializer>::SerializeSeq = serializer.serialize_seq(None)?;
        seq.serialize_element("assert")?;
        seq.serialize_element(condition)?;
        seq.serialize_element(message)?;
        seq.end()
      }
      Self::Backtick { contents, .. } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("evaluate")?;
        seq.serialize_element(contents)?;
        seq.end()
      }
      Self::Call { name, arguments } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("call")?;
        seq.serialize_element(name)?;
        for argument in arguments {
          seq.serialize_element(argument)?;
        }
        seq.end()
      }
      Self::Comparison {
        lhs, operator, rhs, ..
      } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element(&operator.to_string())?;
        seq.serialize_element(lhs)?;
        seq.serialize_element(rhs)?;
        seq.end()
      }
      Self::Concatenation { lhs, rhs, .. } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("concatenate")?;
        seq.serialize_element(lhs)?;
        seq.serialize_element(rhs)?;
        seq.end()
      }
      Self::ListConcatenation { lhs, rhs, .. } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("list-concatenate")?;
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
      Self::Join { lhs, rhs, .. } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("join")?;
        seq.serialize_element(lhs)?;
        seq.serialize_element(rhs)?;
        seq.end()
      }
      Self::List { elements, .. } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("list")?;
        for element in elements {
          seq.serialize_element(element)?;
        }
        seq.end()
      }
      Self::Not { operand } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("not")?;
        seq.serialize_element(operand)?;
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
