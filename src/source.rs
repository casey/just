use super::*;

pub(crate) struct Source<'src> {
  pub(crate) file_depth: u32,
  pub(crate) namepath: Namepath<'src>,
  pub(crate) path: PathBuf,
  pub(crate) submodule_depth: u32,
  pub(crate) working_directory: PathBuf,
}

impl<'src> Source<'src> {
  pub(crate) fn root(path: &Path) -> Self {
    Self {
      file_depth: 0,
      namepath: Namepath::default(),
      path: path.into(),
      submodule_depth: 0,
      working_directory: path.parent().unwrap().into(),
    }
  }

  pub(crate) fn import(&self, path: PathBuf) -> Self {
    Self {
      file_depth: self.file_depth + 1,
      namepath: self.namepath.clone(),
      path,
      submodule_depth: self.submodule_depth,
      working_directory: self.working_directory.clone(),
    }
  }

  pub(crate) fn module(&self, name: Name<'src>, path: PathBuf) -> Self {
    Self {
      file_depth: self.file_depth + 1,
      namepath: self.namepath.join(name),
      submodule_depth: self.submodule_depth + 1,
      working_directory: path.parent().unwrap().into(),
      path,
    }
  }
}
