use super::*;

pub(crate) struct Loader {
  srcs: Arena<String>,
  paths: Arena<PathBuf>,
}

impl Loader {
  pub(crate) fn new() -> Self {
    Loader {
      srcs: Arena::new(),
      paths: Arena::new(),
    }
  }

  pub(crate) fn load<'src>(
    &'src self,
    root: &Path,
    path: &Path,
  ) -> RunResult<(&'src Path, &'src str)> {
    let src = fs::read_to_string(path).map_err(|io_error| Error::Load {
      path: path.to_owned(),
      io_error,
    })?;

    let relative = if let Ok(path) = path.strip_prefix(root.parent().unwrap()) {
      path
    } else {
      path
    };

    Ok((self.paths.alloc(relative.into()), self.srcs.alloc(src)))
  }
}
