use super::*;

pub(crate) struct Loader {
  arena: Arena<String>,
}

impl Loader {
  pub(crate) fn new() -> Self {
    Loader {
      arena: Arena::new(),
    }
  }

  pub(crate) fn load<'src>(&'src self, path: &Path) -> RunResult<&'src str> {
    let src = fs::read_to_string(path).map_err(|io_error| Error::Load {
      path: path.to_owned(),
      io_error,
    })?;
    Ok(self.arena.alloc(src))
  }
}
