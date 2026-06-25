use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum ConstEvalError<'src> {
  Assert {
    message: String,
    name: Name<'src>,
  },
  Const(ConstError<'src>),
  EmptyInterpreter {
    setting: Name<'src>,
  },
  ListInStringContext {
    context: StringContext<'src>,
    value: Value,
  },
  ListOperation {
    lhs: Value,
    operator: ListOperator,
    rhs: Value,
    token: Token<'src>,
  },
  RegexCompile {
    source: regex::Error,
    token: Token<'src>,
  },
}

impl<'src> ConstEvalError<'src> {
  pub(crate) fn context(&self) -> Token<'src> {
    match self {
      Self::Assert { name, .. } => name.token,
      Self::Const(const_error) => const_error.context(),
      Self::EmptyInterpreter { setting } => setting.token,
      Self::ListInStringContext { context, .. } => context.token(),
      Self::ListOperation { token, .. } | Self::RegexCompile { token, .. } => *token,
    }
  }

  pub(crate) fn into_compile_error(self) -> CompileError<'src> {
    self.context().error(CompileErrorKind::ConstEval(self))
  }

  pub(crate) fn assert_const_eval_error(error: Error<'src>) -> Self {
    match error {
      Error::Assert { message, name } => Self::Assert { message, name },
      Error::Const { const_error } => Self::Const(const_error),
      Error::ListInStringContext { context, value } => Self::ListInStringContext { context, value },
      Error::ListOperation {
        lhs,
        operator,
        rhs,
        token,
      } => Self::ListOperation {
        lhs,
        operator,
        rhs,
        token: *token,
      },
      Error::RegexCompile { source, token } => Self::RegexCompile { source, token },
      error => unreachable!("non-const error in const evaluation: {error:?}"),
    }
  }
}

impl Display for ConstEvalError<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Assert { message, .. } => write!(f, "assert failed: {message}"),
      Self::Const(const_error) => write!(f, "{const_error}"),
      Self::EmptyInterpreter { setting } => write!(
        f,
        "`{setting}` setting requires at least one element but evaluated to empty list"
      ),
      Self::ListInStringContext { context, value } => {
        write!(
          f,
          "list value {} {context}",
          value.color_display(Color::never())
        )?;

        if matches!(context, StringContext::Function { .. }) {
          write!(
            f,
            "\nthe behavior of lists with many built-in functions is undecided\n\
            see https://github.com/casey/just#lists",
          )?;
        }

        Ok(())
      }
      Self::ListOperation {
        operator, lhs, rhs, ..
      } => {
        if lhs.is_empty() || rhs.is_empty() {
          write!(f, "operator `{operator}` cannot be applied to empty lists")
        } else {
          write!(
            f,
            "operator `{operator}` cannot be applied to lists of different lengths: {} {operator} {}",
            lhs.color_display(Color::never()),
            rhs.color_display(Color::never()),
          )
        }
      }
      Self::RegexCompile { source, .. } => write!(f, "{source}"),
    }
  }
}
