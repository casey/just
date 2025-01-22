use super::*;

pub(crate) fn which(working_directory: &Path, s: &str) -> Result<Option<String>, String> {
  let cmd = Path::new(s);

  let candidates = match cmd.components().count() {
    0 => return Err("empty command".into()),
    1 => {
      // cmd is a regular command
      let path_var = env::var_os("PATH").ok_or("Environment variable `PATH` is not set")?;
      env::split_paths(&path_var)
        .map(|path| path.join(cmd))
        .collect()
    }
    _ => {
      // cmd contains a path separator, treat it as a path
      vec![cmd.into()]
    }
  };

  for mut candidate in candidates {
    if candidate.is_relative() {
      // This candidate is a relative path, either because the user invoked `which("rel/path")`,
      // or because there was a relative path in `PATH`. Resolve it to an absolute path,
      // relative to the working directory of the just invocation.
      candidate = working_directory.join(candidate);
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
