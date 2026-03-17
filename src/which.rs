use super::*;

pub(crate) fn which(context: function::Context, name: &str) -> Result<Option<String>, String> {
  let name = Path::new(name);

  let paths = match name.components().count() {
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

  for mut path in paths {
    if path.is_relative() {
      // This candidate is a relative path, either because the user invoked `which("rel/path")`,
      // or because there was a relative path in `PATH`. Resolve it to an absolute path,
      // relative to the working directory of the just invocation.
      path = context.execution_context.working_directory().join(path);
    }

    path = path.lexiclean();

    #[allow(unused_mut)]
    let mut candidates = vec![path.clone()];

    #[cfg(windows)]
    if path.extension().is_none() {
      if let Some(pathext) = env::var_os("PATHEXT") {
        let pathext = pathext.to_str().ok_or_else(|| {
          format!(
            "`PATHEXT` environment variable is not valid unicode: {}",
            pathext.to_string_lossy(),
          )
        })?;

        for extension in pathext.split(';') {
          let extension = extension
            .strip_prefix('.')
            .ok_or_else(|| format!("`PATHEXT` entry `{extension}` does not start with `.`"))?;
          candidates.push(path.with_extension(extension));
        }
      }
    }

    for candidate in candidates {
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
  }

  Ok(None)
}
