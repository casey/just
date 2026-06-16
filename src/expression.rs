use super::*;

/// An expression. Note that the Just language grammar has both an `expression`
/// production of additions (`a + b`) and values, and a `value` production of
/// all other value types (for example strings, function calls, and
/// parenthetical groups).
///
/// The parser parses both values and expressions into `Expression`s.
#[derive(PartialEq, Eq, Debug, Clone, Ord, PartialOrd)]
pub(crate) enum Expression<'src> {
  /// `assert(condition, error)`
  Assert {
    name: Name<'src>,
    condition: Box<Self>,
    error: Box<Self>,
  },
  /// `contents`
  Backtick {
    contents: String,
    token: Token<'src>,
  },
  /// `lhs operator rhs`
  Binary {
    operator: BinaryOperator,
    operator_token: Token<'src>,
    lhs: Box<Self>,
    rhs: Box<Self>,
  },
  /// `name(arguments)`
  Call {
    name: Name<'src>,
    arguments: Vec<Expression<'src>>,
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
  /// `[a, b, c]`
  List {
    elements: Vec<Expression<'src>>,
    open: Token<'src>,
  },
  /// `"string_literal"` or `'string_literal'`
  StringLiteral { string_literal: StringLiteral<'src> },
  /// `operator operand`
  Unary {
    operator: UnaryOperator,
    operator_token: Token<'src>,
    operand: Box<Self>,
  },
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
      Self::Assert {
        condition, error, ..
      } => write!(f, "assert({condition}, {error})"),
      Self::Backtick { token, .. } => write!(f, "{}", token.lexeme()),
      Self::Binary {
        operator, lhs, rhs, ..
      } => write!(f, "{lhs} {operator} {rhs}"),
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
      Self::Unary {
        operator: UnaryOperator::Not,
        operand,
        ..
      } => write!(f, "!{operand}"),
      Self::Unary {
        operator: UnaryOperator::Slash,
        operand,
        ..
      } => write!(f, "/ {operand}"),
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
      Self::Binary {
        operator, lhs, rhs, ..
      } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element(operator.serialization())?;
        seq.serialize_element(lhs)?;
        seq.serialize_element(rhs)?;
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
      Self::List { elements, .. } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element("list")?;
        for element in elements {
          seq.serialize_element(element)?;
        }
        seq.end()
      }
      Self::Unary {
        operator, operand, ..
      } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element(operator.serialization())?;
        seq.serialize_element(operand)?;
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
