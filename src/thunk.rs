use super::*;

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq = "feature_allow_slow_enum")]
pub(crate) enum Thunk<'src> {
  Nullary {
    name: Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(function::Context) -> Result<String, String>,
  },
  Unary {
    name: Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(function::Context, &str) -> Result<String, String>,
    arg: Box<Expression<'src>>,
  },
  UnaryOpt {
    name: Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(function::Context, &str, Option<&str>) -> Result<String, String>,
    args: (Box<Expression<'src>>, Box<Option<Expression<'src>>>),
  },
  UnaryPlus {
    name: Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(function::Context, &str, &[String]) -> Result<String, String>,
    args: (Box<Expression<'src>>, Vec<Expression<'src>>),
  },
  Binary {
    name: Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(function::Context, &str, &str) -> Result<String, String>,
    args: [Box<Expression<'src>>; 2],
  },
  BinaryPlus {
    name: Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(function::Context, &str, &str, &[String]) -> Result<String, String>,
    args: ([Box<Expression<'src>>; 2], Vec<Expression<'src>>),
  },
  Ternary {
    name: Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(function::Context, &str, &str, &str) -> Result<String, String>,
    args: [Box<Expression<'src>>; 3],
  },
  BinaryUInteger {
    name: Name<'src>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    function: fn(function::Context, &str, u16) -> Result<String, String>,
    args: [Box<Expression<'src>>; 2],
  },
}

impl<'src> Thunk<'src> {
  pub(crate) fn name(&self) -> Name<'src> {
    match self {
      Self::Nullary { name, .. }
      | Self::Unary { name, .. }
      | Self::UnaryOpt { name, .. }
      | Self::UnaryPlus { name, .. }
      | Self::Binary { name, .. }
      | Self::BinaryUInteger { name, .. }
      | Self::BinaryPlus { name, .. }
      | Self::Ternary { name, .. } => *name,
    }
  }

  pub(crate) fn resolve(
    name: Name<'src>,
    mut arguments: Vec<Expression<'src>>,
  ) -> CompileResult<'src, Thunk<'src>> {
    function::get(name.lexeme()).map_or(
      Err(name.error(CompileErrorKind::UnknownFunction {
        function: name.lexeme(),
      })),
      |function| match (function, arguments.len()) {
        (Function::Nullary(function), 0) => Ok(Thunk::Nullary { function, name }),
        (Function::Unary(function), 1) => Ok(Thunk::Unary {
          function,
          arg: arguments.pop().unwrap().into(),
          name,
        }),
        (Function::UnaryOpt(function), 1..=2) => {
          let a = arguments.remove(0).into();
          let b = match arguments.pop() {
            Some(value) => Some(value).into(),
            None => None.into(),
          };
          Ok(Thunk::UnaryOpt {
            function,
            args: (a, b),
            name,
          })
        }
        (Function::UnaryPlus(function), 1..=usize::MAX) => {
          let rest = arguments.drain(1..).collect();
          let a = Box::new(arguments.pop().unwrap());
          Ok(Thunk::UnaryPlus {
            function,
            args: (a, rest),
            name,
          })
        }
        (Function::Binary(function), 2) => {
          let b = arguments.pop().unwrap().into();
          let a = arguments.pop().unwrap().into();
          Ok(Thunk::Binary {
            function,
            args: [a, b],
            name,
          })
        }
        (Function::BinaryPlus(function), 2..=usize::MAX) => {
          let rest = arguments.drain(2..).collect();
          let b = arguments.pop().unwrap().into();
          let a = arguments.pop().unwrap().into();
          Ok(Thunk::BinaryPlus {
            function,
            args: ([a, b], rest),
            name,
          })
        }
        (Function::Ternary(function), 3) => {
          let c = arguments.pop().unwrap().into();
          let b = arguments.pop().unwrap().into();
          let a = arguments.pop().unwrap().into();
          Ok(Thunk::Ternary {
            function,
            args: [a, b, c],
            name,
          })
        }
        (Function::BinaryUInteger(function), 2) => {
          let b = arguments.pop().unwrap().into();
          let a = arguments.pop().unwrap().into();
          Ok(Thunk::BinaryUInteger {
            function,
            args: [a, b],
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
      Unary { name, arg, .. } => write!(f, "{}({arg})", name.lexeme()),
      UnaryOpt {
        name, args: (a, b), ..
      } => {
        if let Some(b) = b.as_ref() {
          write!(f, "{}({a}, {b})", name.lexeme())
        } else {
          write!(f, "{}({a})", name.lexeme())
        }
      }
      UnaryPlus {
        name,
        args: (a, rest),
        ..
      } => {
        write!(f, "{}({a}", name.lexeme())?;
        for arg in rest {
          write!(f, ", {arg}")?;
        }
        write!(f, ")")
      }
      Binary {
        name, args: [a, b], ..
      } => write!(f, "{}({a}, {b})", name.lexeme()),
      BinaryUInteger {
        name, args: [a, b], ..
      } => write!(f, "{}({a}, {b})", name.lexeme()),
      BinaryPlus {
        name,
        args: ([a, b], rest),
        ..
      } => {
        write!(f, "{}({a}, {b}", name.lexeme())?;
        for arg in rest {
          write!(f, ", {arg}")?;
        }
        write!(f, ")")
      }
      Ternary {
        name,
        args: [a, b, c],
        ..
      } => write!(f, "{}({a}, {b}, {c})", name.lexeme()),
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
    seq.serialize_element(&self.name())?;
    match self {
      Self::Nullary { .. } => {}
      Self::Unary { arg, .. } => seq.serialize_element(&arg)?,
      Self::UnaryOpt {
        args: (a, opt_b), ..
      } => {
        seq.serialize_element(a)?;
        if let Some(b) = opt_b.as_ref() {
          seq.serialize_element(b)?;
        }
      }
      Self::UnaryPlus {
        args: (a, rest), ..
      } => {
        seq.serialize_element(a)?;
        for arg in rest {
          seq.serialize_element(arg)?;
        }
      }
      Self::Binary { args, .. } => {
        for arg in args {
          seq.serialize_element(arg)?;
        }
      }
      Self::BinaryUInteger { args, .. } => {
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
