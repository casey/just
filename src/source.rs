use super::*;

#[derive(Debug)]
pub(crate) struct Source {
  pub(crate) file_depth: u32,
  pub(crate) file_path: Vec<PathBuf>,
  pub(crate) import_offsets: Vec<usize>,
  pub(crate) module_path: Modulepath,
  pub(crate) path: PathBuf,
  pub(crate) working_directory: PathBuf,
}

impl Source {
  pub(crate) fn root(path: &Path) -> Self {
    Self {
      file_depth: 0,
      file_path: vec![path.into()],
      import_offsets: Vec::new(),
      module_path: Modulepath::default(),
      path: path.into(),
      working_directory: path.parent().unwrap().into(),
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
      import_offsets: self
        .import_offsets
        .iter()
        .copied()
        .chain(iter::once(import_offset))
        .collect(),
      module_path: self.module_path.clone(),
      path,
      working_directory: self.working_directory.clone(),
    }
  }

  pub(crate) fn module(&self, name: &str, path: PathBuf) -> Self {
    Self {
      file_depth: self.file_depth + 1,
      file_path: self
        .file_path
        .clone()
        .into_iter()
        .chain(iter::once(path.clone()))
        .collect(),
      import_offsets: Vec::new(),
      module_path: self.module_path.join(name),
      path: path.clone(),
      working_directory: path.parent().unwrap().into(),
    }
  }
}
