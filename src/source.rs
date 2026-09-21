use super::*;

#[derive(Debug)]
pub(crate) struct Source<'src> {
  pub(crate) file_depth: u32,
  pub(crate) file_path: Vec<Utf8PathBuf>,
  pub(crate) import_offsets: Vec<usize>,
  pub(crate) namepath: Option<Namepath<'src>>,
  pub(crate) path: Utf8PathBuf,
  pub(crate) working_directory: Utf8PathBuf,
}

impl<'src> Source<'src> {
  pub(crate) fn root(path: &Utf8Path) -> Self {
    Self {
      file_depth: 0,
      file_path: vec![path.into()],
      import_offsets: Vec::new(),
      namepath: None,
      path: path.into(),
      working_directory: path.parent().unwrap().into(),
    }
  }

  pub(crate) fn import(&self, path: Utf8PathBuf, import_offset: usize) -> Self {
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
      namepath: self.namepath.clone(),
      path,
      working_directory: self.working_directory.clone(),
    }
  }

  pub(crate) fn module(&self, name: Name<'src>, path: Utf8PathBuf) -> Self {
    Self {
      file_depth: self.file_depth + 1,
      file_path: self
        .file_path
        .clone()
        .into_iter()
        .chain(iter::once(path.clone()))
        .collect(),
      import_offsets: Vec::new(),
      namepath: Some(
        self
          .namepath
          .as_ref()
          .map_or_else(|| name.into(), |namepath| namepath.join(name)),
      ),
      path: path.clone(),
      working_directory: path.parent().unwrap().into(),
    }
  }
}
