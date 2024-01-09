use super::*;

pub(crate) struct Source<'src> {
  pub(crate) path: PathBuf,
  pub(crate) depth: u32,
  pub(crate) namepath: Namepath<'src>,
}

impl<'src> Source<'src> {
  pub(crate) fn root(path: &Path) -> Self {
    Self {
      path: path.into(),
      depth: 0,
      namepath: Namepath::default(),
    }
  }

  pub(crate) fn import(&self, path: PathBuf) -> Self {
    Self {
      depth: self.depth + 1,
      path,
      namepath: self.namepath.clone(),
    }
  }

  pub(crate) fn module(&self, name: Name<'src>, path: PathBuf) -> Self {
    Self {
      path,
      depth: self.depth + 1,
      namepath: self.namepath.join(name),
    }
  }
}
