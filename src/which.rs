use super::*;

/// Resolve an executable name to an absolute path using PATH.
///
/// On Windows, also checks PATHEXT extensions (.exe, .bat, .cmd, etc.)
/// This function respects PATH order, unlike Windows' `CreateProcess` which
/// prioritizes System32.
pub(crate) fn resolve_executable(
  name: &str,
  working_directory: &Path,
) -> RunResult<'static, PathBuf> {
  let name = Path::new(name);

  // If already absolute, validate it exists and return as-is
  if name.is_absolute() {
    let candidate = name.lexiclean();
    if is_executable::is_executable(&candidate) {
      return Ok(candidate);
    }
    return Err(Error::ExecutableNotFound {
      name: name.to_string_lossy().to_string(),
      suggestion: None,
    });
  }

  // Check if it's a relative path (contains path separators)
  let candidates = if name.components().count() > 1 {
    // Relative path - resolve relative to working directory
    vec![working_directory.join(name)]
  } else {
    // Simple command name - search PATH
    #[allow(unused_mut)] // mut is needed on Windows for PATHEXT extensions
    let mut candidates: Vec<PathBuf> = env::split_paths(
      &env::var_os("PATH")
        .ok_or_else(|| Error::internal("PATH environment variable not set"))?,
    )
    .map(|path| path.join(name))
    .collect();

    // On Windows, also try with PATHEXT extensions
    #[cfg(windows)]
    {
      let pathext = env::var_os("PATHEXT")
        .unwrap_or_else(|| OsString::from(".EXE;.COM;.BAT;.CMD;.VBS;.VBE;.JS;.JSE;.WSF;.WSH"));

      let extensions: Vec<String> = env::split_paths(&pathext)
        .filter_map(|ext| ext.to_str().map(|s| s.to_string()))
        .collect();

      // For each PATH entry, try each extension
      for path in env::split_paths(&env::var_os("PATH").unwrap()) {
        for ext in &extensions {
          candidates.push(path.join(format!("{}{}", name.to_string_lossy(), ext)));
        }
      }
    }

    candidates
  };

  // Try each candidate
  for mut candidate in candidates {
    if candidate.is_relative() {
      candidate = working_directory.join(candidate);
    }

    candidate = candidate.lexiclean();

    if is_executable::is_executable(&candidate) {
      return Ok(candidate);
    }
  }

  // Not found - provide helpful error
  Err(Error::ExecutableNotFound {
    name: name.to_string_lossy().to_string(),
    suggestion: None,
  })
}

pub(crate) fn which(context: function::Context, name: &str) -> Result<Option<String>, String> {
  let name = Path::new(name);

  let candidates = match name.components().count() {
    0 => return Err("empty command".into()),
    1 => {
      // cmd is a regular command
      env::split_paths(&env::var_os("PATH").ok_or("`PATH` environment variable not set")?)
        .map(|path| path.join(name))
        .collect()
    }
    _ => {
      // cmd contains a path separator, treat it as a path
      vec![name.into()]
    }
  };

  for mut candidate in candidates {
    if candidate.is_relative() {
      // This candidate is a relative path, either because the user invoked `which("rel/path")`,
      // or because there was a relative path in `PATH`. Resolve it to an absolute path,
      // relative to the working directory of the just invocation.
      candidate = context
        .evaluator
        .context
        .working_directory()
        .join(candidate);
    }

    candidate = candidate.lexiclean();

    if is_executable::is_executable(&candidate) {
      return candidate
        .to_str()
        .map(|candidate| Some(candidate.into()))
        .ok_or_else(|| {
          format!(
            "Executable path is not valid unicode: {}",
            candidate.display()
          )
        });
    }
  }

  Ok(None)
}
