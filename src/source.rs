use super::*;

#[derive(Debug)]
pub(crate) struct Source<'src> {
  pub(crate) file_depth: u32,
  pub(crate) file_path: Vec<PathBuf>,
  pub(crate) import_offsets: Vec<usize>,
  pub(crate) namepath: Namepath<'src>,
  pub(crate) path: PathBuf,
  pub(crate) submodule_depth: u32,
  pub(crate) working_directory: PathBuf,
}

impl<'src> Source<'src> {
  pub(crate) fn root(path: &Path) -> Self {
    Self {
      file_depth: 0,
      file_path: vec![path.into()],
      namepath: Namepath::default(),
      path: path.into(),
      submodule_depth: 0,
      working_directory: path.parent().unwrap().into(),
      import_offsets: Vec::new(),
    }
  }

  pub(crate) fn import(&self, path: PathBuf, import_offset: usize) -> Self {
    Self {
      file_depth: self.file_depth + 1,
      file_path: self
        .file_path
        .clone()
        .into_iter()
        .chain(iter::once(path.clone()))
        .collect(),
      namepath: self.namepath.clone(),
      path,
      submodule_depth: self.submodule_depth,
      working_directory: self.working_directory.clone(),
      import_offsets: self
        .import_offsets
        .iter()
        .cloned()
        .chain(iter::once(import_offset))
        .collect(),
    }
  }

  pub(crate) fn module(&self, name: Name<'src>, path: PathBuf) -> Self {
    Self {
      file_depth: self.file_depth + 1,
      file_path: self
        .file_path
        .clone()
        .into_iter()
        .chain(iter::once(path.clone()))
        .collect(),
      namepath: self.namepath.join(name),
      path: path.clone(),
      submodule_depth: self.submodule_depth + 1,
      working_directory: path.parent().unwrap().into(),
      import_offsets: Vec::new(),
    }
  }
}
