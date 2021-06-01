use crate::common::*;

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq = "feature_allow_slow_enum")]
pub(crate) enum Thunk<'src> {
  Nullary {
    name:     Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(&FunctionContext) -> Result<String, String>,
  },
  Unary {
    name:     Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(&FunctionContext, &str) -> Result<String, String>,
    arg:      Box<Expression<'src>>,
  },
  Binary {
    name:     Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(&FunctionContext, &str, &str) -> Result<String, String>,
    args:     [Box<Expression<'src>>; 2],
  },
}

impl<'src> Thunk<'src> {
  pub(crate) fn resolve(
    name: Name<'src>,
    mut arguments: Vec<Expression<'src>>,
  ) -> CompilationResult<'src, Thunk<'src>> {
    if let Some(function) = crate::function::TABLE.get(&name.lexeme()) {
      match (function, arguments.len()) {
        (Function::Nullary(function), 0) => Ok(Thunk::Nullary {
          function: *function,
          name,
        }),
        (Function::Unary(function), 1) => Ok(Thunk::Unary {
          function: *function,
          arg: Box::new(arguments.pop().unwrap()),
          name,
        }),
        (Function::Binary(function), 2) => {
          let b = Box::new(arguments.pop().unwrap());
          let a = Box::new(arguments.pop().unwrap());
          Ok(Thunk::Binary {
            function: *function,
            args: [a, b],
            name,
          })
        },
        _ => Err(
          name.error(CompilationErrorKind::FunctionArgumentCountMismatch {
            function: name.lexeme(),
            found:    arguments.len(),
            expected: function.argc(),
          }),
        ),
      }
    } else {
      Err(name.error(CompilationErrorKind::UnknownFunction {
        function: name.lexeme(),
      }))
    }
  }
}

impl Display for Thunk<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use Thunk::*;
    match self {
      Nullary { name, .. } => write!(f, "{}()", name.lexeme()),
      Unary { name, arg, .. } => write!(f, "{}({})", name.lexeme(), arg),
      Binary {
        name, args: [a, b], ..
      } => write!(f, "{}({}, {})", name.lexeme(), a, b),
    }
  }
}
