use super::*;

pub(crate) struct Loader {
  paths: Arena<PathBuf>,
  srcs: Arena<String>,
}

impl Loader {
  pub(crate) fn new() -> Self {
    Self {
      srcs: Arena::new(),
      paths: Arena::new(),
    }
  }

  pub(crate) fn load<'src>(
    &'src self,
    root: &Path,
    path: &Path,
  ) -> RunResult<'src, (&'src Path, &'src str)> {
    let src = fs::read_to_string(path).map_err(|io_error| Error::Load {
      path: path.into(),
      io_error,
    })?;

    let relative = path.strip_prefix(root.parent().unwrap()).unwrap_or(path);

    Ok((self.paths.alloc(relative.into()), self.srcs.alloc(src)))
  }
}
