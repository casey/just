use common::*;
use target;

lazy_static! {
  static ref FUNCTIONS: Map<&'static str, Function> = vec![
    ("arch",               Function::Nullary(arch              )),
    ("os",                 Function::Nullary(os                )),
    ("os_family",          Function::Nullary(os_family         )),
    ("env_var",            Function::Unary  (env_var           )),
    ("env_var_or_default", Function::Binary (env_var_or_default)),
  ].into_iter().collect();
}

enum Function {
  Nullary(fn(          ) -> Result<String, String>),
  Unary  (fn(&str      ) -> Result<String, String>),
  Binary (fn(&str, &str) -> Result<String, String>),
}

impl Function {
  fn argc(&self) -> usize {
    use self::Function::*;
    match *self {
      Nullary(_) => 0,
      Unary(_)   => 1,
      Binary(_)  => 2,
    }
  }
}

pub fn resolve_function<'a>(token: &Token<'a>, argc: usize) -> CompilationResult<'a, ()> {
  let name = token.lexeme;
  if let Some(function) = FUNCTIONS.get(&name) {
    use self::Function::*;
    match (function, argc) {
      (&Nullary(_), 0) => Ok(()),
      (&Unary(_),   1) => Ok(()),
      (&Binary(_),  2) => Ok(()),
      _               => {
        Err(token.error(CompilationErrorKind::FunctionArgumentCountMismatch{
          function: name, found: argc, expected: function.argc(),
        }))
      }
    }
  } else {
    Err(token.error(CompilationErrorKind::UnknownFunction{function: token.lexeme}))
  }
}

pub fn evaluate_function<'a>(token: &Token<'a>, name: &'a str, arguments: &[String]) -> RunResult<'a, String> {
  if let Some(function) = FUNCTIONS.get(name) {
    use self::Function::*;
    let argc = arguments.len();
    match (function, argc) {
      (&Nullary(f), 0) => f()
        .map_err(|message| RuntimeError::FunctionCall{token: token.clone(), message}),
      (&Unary(f), 1) => f(&arguments[0])
        .map_err(|message| RuntimeError::FunctionCall{token: token.clone(), message}),
      (&Binary(f), 2) => f(&arguments[0], &arguments[1])
        .map_err(|message| RuntimeError::FunctionCall{token: token.clone(), message}),
      _                => {
        Err(RuntimeError::Internal {
          message: format!("attempted to evaluate function `{}` with {} arguments", name, argc)
        })
      }
    }
  } else {
    Err(RuntimeError::Internal {
      message: format!("attempted to evaluate unknown function: `{}`", name)
    })
  }
}

pub fn arch() -> Result<String, String> {
  Ok(target::arch().to_string())
}

pub fn os() -> Result<String, String> {
  Ok(target::os().to_string())
}

pub fn os_family() -> Result<String, String> {
  Ok(target::os_family().to_string())
}

pub fn env_var<'a>(key: &str) -> Result<String, String> {
  use std::env::VarError::*;
  match env::var(key) {
    Err(NotPresent) => Err(format!("environment variable `{}` not present", key)),
    Err(NotUnicode(os_string)) => 
      Err(format!("environment variable `{}` not unicode: {:?}", key, os_string)),
    Ok(value) => Ok(value),
  }
}

pub fn env_var_or_default<'a>(key: &str, default: &str) -> Result<String, String> {
  use std::env::VarError::*;
  match env::var(key) {
    Err(NotPresent) => Ok(default.to_string()),
    Err(NotUnicode(os_string)) => 
      Err(format!("environment variable `{}` not unicode: {:?}", key, os_string)),
    Ok(value) => Ok(value),
  }
}
