use crate::common::*;

use Function::*;
pub(crate) enum Function {
  Nullary(fn(&FunctionContext) -> Result<String, String>),
  Unary(fn(&FunctionContext, &str) -> Result<String, String>),
  Binary(fn(&FunctionContext, &str, &str) -> Result<String, String>),
}

lazy_static! {
  pub(crate) static ref TABLE: BTreeMap<&'static str, Function> = vec![
    ("arch", Nullary(arch)),
    ("os", Nullary(os)),
    ("os_family", Nullary(os_family)),
    ("justfile_directory", Nullary(justfile_directory)),
    ("justfile", Nullary(justfile)),
    ("invocation_directory", Nullary(invocation_directory)),
    ("env_var", Unary(env_var)),
    ("env_var_or_default", Binary(env_var_or_default)),
    ("just_executable", Nullary(just_executable)),
    ("file_name", Unary(file_name)),
    ("parent_directory", Unary(parent_directory)),
    ("file_stem", Unary(file_stem)),
    ("without_extension", Unary(without_extension)),
    ("extension", Unary(extension))
  ]
  .into_iter()
  .collect();
}

impl Function {
  pub(crate) fn argc(&self) -> usize {
    match *self {
      Nullary(_) => 0,
      Unary(_) => 1,
      Binary(_) => 2,
    }
  }
}

fn arch(_context: &FunctionContext) -> Result<String, String> {
  Ok(target::arch().to_owned())
}

fn os(_context: &FunctionContext) -> Result<String, String> {
  Ok(target::os().to_owned())
}

fn os_family(_context: &FunctionContext) -> Result<String, String> {
  Ok(target::os_family().to_owned())
}

fn invocation_directory(context: &FunctionContext) -> Result<String, String> {
  Platform::convert_native_path(
    &context.search.working_directory,
    context.invocation_directory,
  )
  .map_err(|e| format!("Error getting shell path: {}", e))
}

fn justfile(context: &FunctionContext) -> Result<String, String> {
  context
    .search
    .justfile
    .to_str()
    .map(str::to_owned)
    .ok_or_else(|| {
      format!(
        "Justfile path is not valid unicode: {}",
        context.search.justfile.to_string_lossy()
      )
    })
}

fn justfile_directory(context: &FunctionContext) -> Result<String, String> {
  let justfile_directory = context.search.justfile.parent().ok_or_else(|| {
    format!(
      "Could not resolve justfile directory. Justfile `{}` had no parent.",
      context.search.justfile.display()
    )
  })?;

  justfile_directory
    .to_str()
    .map(str::to_owned)
    .ok_or_else(|| {
      format!(
        "Justfile directory is not valid unicode: {}",
        justfile_directory.to_string_lossy()
      )
    })
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
  use std::env::VarError::*;

  if let Some(value) = context.dotenv.get(key) {
    return Ok(value.clone());
  }

  match env::var(key) {
    Err(NotPresent) => Ok(default.to_owned()),
    Err(NotUnicode(os_string)) => Err(format!(
      "environment variable `{}` not unicode: {:?}",
      key, os_string
    )),
    Ok(value) => Ok(value),
  }
}

fn just_executable(_context: &FunctionContext) -> Result<String, String> {
  let exe_path =
    std::env::current_exe().map_err(|e| format!("Error getting current executable: {}", e))?;

  exe_path.to_str().map(str::to_owned).ok_or_else(|| {
    format!(
      "Executable path is not valid unicode: {}",
      exe_path.to_string_lossy()
    )
  })
}

fn file_name(_context: &FunctionContext, path: &str) -> Result<String, String> {
  Utf8Path::new(path)
    .file_name()
    .map(str::to_owned)
    .ok_or_else(|| format!("Cannot get file name from `{}`", path))
}

fn parent_directory(_context: &FunctionContext, path: &str) -> Result<String, String> {
  Utf8Path::new(path)
    .parent()
    .map(Utf8Path::to_string)
    .ok_or_else(|| format!("Cannot get parent directory from `{}`", path))
}

fn file_stem(_context: &FunctionContext, path: &str) -> Result<String, String> {
  Utf8Path::new(path)
    .file_stem()
    .map(str::to_owned)
    .ok_or_else(|| format!("Cannot get file stem from `{}`", path))
}

fn without_extension(_context: &FunctionContext, path: &str) -> Result<String, String> {
  let path = Utf8Path::new(path);

  let parent = path
    .parent()
    .ok_or_else(|| format!("Cannot extract parent from `{}`", path))?;

  let file_stem = path
    .file_stem()
    .ok_or_else(|| format!("Cannot extract file_stem from `{}`", path))?;

  Ok(parent.join(file_stem).to_string())
}

fn extension(_context: &FunctionContext, path: &str) -> Result<String, String> {
  Utf8Path::new(path)
    .extension()
    .map(str::to_owned)
    .ok_or_else(|| format!("Cannot get extension from `{}`", path))
}
