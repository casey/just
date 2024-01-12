use super::*;

pub(crate) struct Source<'src> {
  pub(crate) path: PathBuf,
  pub(crate) depth: u32,
  pub(crate) namepath: Namepath<'src>,
  pub(crate) working_directory: PathBuf,
}

impl<'src> Source<'src> {
  pub(crate) fn root(path: &Path) -> Self {
    Self {
      path: path.into(),
      depth: 0,
      namepath: Namepath::default(),
      working_directory: path.parent().unwrap().into(),
    }
  }

  pub(crate) fn import(&self, path: PathBuf) -> Self {
    Self {
      depth: self.depth + 1,
      path,
      namepath: self.namepath.clone(),
      working_directory: self.working_directory.clone(),
    }
  }

  pub(crate) fn module(&self, name: Name<'src>, path: PathBuf) -> Self {
    Self {
      working_directory: path.parent().unwrap().into(),
      path,
      depth: self.depth + 1,
      namepath: self.namepath.join(name),
    }
  }
}
