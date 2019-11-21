use crate::common::*;

use target;

pub(crate) enum Function {
  Nullary(fn(&FunctionContext) -> Result<String, String>),
  Unary(fn(&FunctionContext, &str) -> Result<String, String>),
  Binary(fn(&FunctionContext, &str, &str) -> Result<String, String>),
}

lazy_static! {
  pub(crate) static ref TABLE: BTreeMap<&'static str, Function> = vec![
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

impl Function {
  pub(crate) fn argc(&self) -> usize {
    use self::Function::*;
    match *self {
      Nullary(_) => 0,
      Unary(_) => 1,
      Binary(_) => 2,
    }
  }
}

fn arch(_context: &FunctionContext) -> Result<String, String> {
  Ok(target::arch().to_string())
}

fn os(_context: &FunctionContext) -> Result<String, String> {
  Ok(target::os().to_string())
}

fn os_family(_context: &FunctionContext) -> Result<String, String> {
  Ok(target::os_family().to_string())
}

fn invocation_directory(context: &FunctionContext) -> Result<String, String> {
  Platform::to_shell_path(context.working_directory, context.invocation_directory)
    .map_err(|e| format!("Error getting shell path: {}", e))
}

fn env_var(context: &FunctionContext, key: &str) -> Result<String, String> {
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

fn env_var_or_default(
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
