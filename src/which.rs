use super::*;

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
