use common::*;
use target;

pub fn resolve_function<'a>(token: &Token<'a>) -> CompilationResult<'a, ()> {
  if !&["arch", "os", "os_family"].contains(&token.lexeme) {
    Err(token.error(CompilationErrorKind::UnknownFunction{function: token.lexeme}))
  } else {
    Ok(())
  }
}

pub fn evaluate_function<'a>(name: &'a str) -> RunResult<'a, String> {
  match name {
    "arch"      => Ok(arch().to_string()),
    "os"        => Ok(os().to_string()),
    "os_family" => Ok(os_family().to_string()),
    _           => Err(RuntimeError::Internal {
      message: format!("attempted to evaluate unknown function: `{}`", name)
    })
  }
}

pub fn arch() -> &'static str {
  target::arch()
}

pub fn os() -> &'static str {
  target::os()
}

pub fn os_family() -> &'static str {
  target::os_family()
}
