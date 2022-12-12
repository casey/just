use super::*;

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq = "feature_allow_slow_enum")]
pub(crate) enum Thunk<'src> {
  Nullary {
    name: Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(&FunctionContext) -> Result<String, String>,
  },
  Unary {
    name: Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(&FunctionContext, &str) -> Result<String, String>,
    arg: Box<Expression<'src>>,
  },
  Binary {
    name: Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(&FunctionContext, &str, &str) -> Result<String, String>,
    args: [Box<Expression<'src>>; 2],
  },
  BinaryPlus {
    name: Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(&FunctionContext, &str, &str, &[String]) -> Result<String, String>,
    args: ([Box<Expression<'src>>; 2], Vec<Expression<'src>>),
  },
  Ternary {
    name: Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(&FunctionContext, &str, &str, &str) -> Result<String, String>,
    args: [Box<Expression<'src>>; 3],
  },
}

impl<'src> Thunk<'src> {
  fn name(&self) -> &Name<'src> {
    match self {
      Self::Nullary { name, .. }
      | Self::Unary { name, .. }
      | Self::Binary { name, .. }
      | Self::BinaryPlus { name, .. }
      | Self::Ternary { name, .. } => name,
    }
  }

  pub(crate) fn resolve(
    name: Name<'src>,
    mut arguments: Vec<Expression<'src>>,
  ) -> CompileResult<'src, Thunk<'src>> {
    crate::function::get(name.lexeme()).map_or(
      Err(name.error(CompileErrorKind::UnknownFunction {
        function: name.lexeme(),
      })),
      |function| match (function, arguments.len()) {
        (Function::Nullary(function), 0) => Ok(Thunk::Nullary { function, name }),
        (Function::Unary(function), 1) => Ok(Thunk::Unary {
          function,
          arg: Box::new(arguments.pop().unwrap()),
          name,
        }),
        (Function::Binary(function), 2) => {
          let b = Box::new(arguments.pop().unwrap());
          let a = Box::new(arguments.pop().unwrap());
          Ok(Thunk::Binary {
            function,
            args: [a, b],
            name,
          })
        }
        (Function::BinaryPlus(function), 2..=usize::MAX) => {
          let rest = arguments.drain(2..).collect();
          let b = Box::new(arguments.pop().unwrap());
          let a = Box::new(arguments.pop().unwrap());
          Ok(Thunk::BinaryPlus {
            function,
            args: ([a, b], rest),
            name,
          })
        }
        (Function::Ternary(function), 3) => {
          let c = Box::new(arguments.pop().unwrap());
          let b = Box::new(arguments.pop().unwrap());
          let a = Box::new(arguments.pop().unwrap());
          Ok(Thunk::Ternary {
            function,
            args: [a, b, c],
            name,
          })
        }
        (function, _) => Err(name.error(CompileErrorKind::FunctionArgumentCountMismatch {
          function: name.lexeme(),
          found: arguments.len(),
          expected: function.argc(),
        })),
      },
    )
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
      BinaryPlus {
        name,
        args: ([a, b], rest),
        ..
      } => {
        write!(f, "{}({}, {}", name.lexeme(), a, b)?;
        for arg in rest {
          write!(f, ", {}", arg)?;
        }
        write!(f, ")")
      }
      Ternary {
        name,
        args: [a, b, c],
        ..
      } => write!(f, "{}({}, {}, {})", name.lexeme(), a, b, c),
    }
  }
}

impl<'src> Serialize for Thunk<'src> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut seq = serializer.serialize_seq(None)?;
    seq.serialize_element("call")?;
    seq.serialize_element(self.name())?;
    match self {
      Self::Nullary { .. } => {}
      Self::Unary { arg, .. } => seq.serialize_element(&arg)?,
      Self::Binary { args, .. } => {
        for arg in args {
          seq.serialize_element(arg)?;
        }
      }
      Self::BinaryPlus { args, .. } => {
        for arg in args.0.iter().map(Box::as_ref).chain(&args.1) {
          seq.serialize_element(arg)?;
        }
      }
      Self::Ternary { args, .. } => {
        for arg in args {
          seq.serialize_element(arg)?;
        }
      }
    }
    seq.end()
  }
}
