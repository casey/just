use crate::common::*;

use target;

lazy_static! {
  static ref FUNCTIONS: BTreeMap<&'static str, Function> = vec![
    ("arch", Function::Nullary(arch)),
    ("os", Function::Nullary(os)),
    ("os_family", Function::Nullary(os_family)),
    ("env_var", Function::Unary(env_var)),
    ("env_var_or_default", Function::Binary(env_var_or_default)),
    (
      "invocation_directory",
      Function::Nullary(invocation_directory)
    ),
  ]
  .into_iter()
  .collect();
}

pub(crate) enum Function {
  Nullary(fn(&FunctionContext) -> Result<String, String>),
  Unary(fn(&FunctionContext, &str) -> Result<String, String>),
  Binary(fn(&FunctionContext, &str, &str) -> Result<String, String>),
}

impl Function {
  fn argc(&self) -> usize {
    use self::Function::*;
    match *self {
      Nullary(_) => 0,
      Unary(_) => 1,
      Binary(_) => 2,
    }
  }

  pub(crate) fn resolve<'a>(token: &Token<'a>, argc: usize) -> CompilationResult<'a, ()> {
    let name = token.lexeme();
    if let Some(function) = FUNCTIONS.get(&name) {
      use self::Function::*;
      match (function, argc) {
        (&Nullary(_), 0) | (&Unary(_), 1) | (&Binary(_), 2) => Ok(()),
        _ => Err(
          token.error(CompilationErrorKind::FunctionArgumentCountMismatch {
            function: name,
            found: argc,
            expected: function.argc(),
          }),
        ),
      }
    } else {
      Err(token.error(CompilationErrorKind::UnknownFunction {
        function: token.lexeme(),
      }))
    }
  }

  pub(crate) fn evaluate<'a>(
    function_name: Name<'a>,
    context: &FunctionContext,
    arguments: &[String],
  ) -> RunResult<'a, String> {
    let name = function_name.lexeme();
    if let Some(function) = FUNCTIONS.get(name) {
      use self::Function::*;
      let argc = arguments.len();
      match (function, argc) {
        (&Nullary(f), 0) => f(context).map_err(|message| RuntimeError::FunctionCall {
          function: function_name,
          message,
        }),
        (&Unary(f), 1) => f(context, &arguments[0]).map_err(|message| RuntimeError::FunctionCall {
          function: function_name,
          message,
        }),
        (&Binary(f), 2) => {
          f(context, &arguments[0], &arguments[1]).map_err(|message| RuntimeError::FunctionCall {
            function: function_name,
            message,
          })
        }
        _ => Err(RuntimeError::Internal {
          message: format!(
            "attempted to evaluate function `{}` with {} arguments",
            name, argc
          ),
        }),
      }
    } else {
      Err(RuntimeError::Internal {
        message: format!("attempted to evaluate unknown function: `{}`", name),
      })
    }
  }
}

pub(crate) fn arch(_context: &FunctionContext) -> Result<String, String> {
  Ok(target::arch().to_string())
}

pub(crate) fn os(_context: &FunctionContext) -> Result<String, String> {
  Ok(target::os().to_string())
}

pub(crate) fn os_family(_context: &FunctionContext) -> Result<String, String> {
  Ok(target::os_family().to_string())
}

pub(crate) fn invocation_directory(context: &FunctionContext) -> Result<String, String> {
  Platform::to_shell_path(context.working_directory, context.invocation_directory)
    .map_err(|e| format!("Error getting shell path: {}", e))
}

pub(crate) fn env_var(context: &FunctionContext, key: &str) -> Result<String, String> {
  use std::env::VarError::*;

  if let Some(value) = context.dotenv.get(key) {
    return Ok(value.clone());
  }

  match env::var(key) {
    Err(NotPresent) => Err(format!("environment variable `{}` not present", key)),
    Err(NotUnicode(os_string)) => Err(format!(
      "environment variable `{}` not unicode: {:?}",
      key, os_string
    )),
    Ok(value) => Ok(value),
  }
}

pub(crate) fn env_var_or_default(
  context: &FunctionContext,
  key: &str,
  default: &str,
) -> Result<String, String> {
  if let Some(value) = context.dotenv.get(key) {
    return Ok(value.clone());
  }

  use std::env::VarError::*;
  match env::var(key) {
    Err(NotPresent) => Ok(default.to_string()),
    Err(NotUnicode(os_string)) => Err(format!(
      "environment variable `{}` not unicode: {:?}",
      key, os_string
    )),
    Ok(value) => Ok(value),
  }
}
