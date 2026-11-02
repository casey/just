use super::*;

pub(crate) fn metadata_opt(path: &Utf8Path) -> RunResult<'static, Option<fs::Metadata>> {
  match path.metadata() {
    Ok(metadata) => Ok(Some(metadata)),
    Err(source) => {
      if source.kind() == io::ErrorKind::NotFound {
        Ok(None)
      } else {
        Err(Error::FilesystemIo {
          path: path.into(),
          source,
        })
      }
    }
  }
}

pub(crate) fn is_file(path: &Utf8Path) -> RunResult<'static, bool> {
  Ok(metadata_opt(path)?.is_some_and(|metadata| metadata.is_file()))
}
