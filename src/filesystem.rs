use super::*;

pub(crate) fn is_file(path: &Path) -> RunResult<'static, bool> {
  match path.metadata() {
    Ok(metadata) => Ok(metadata.is_file()),
    Err(source) => {
      if source.kind() == io::ErrorKind::NotFound {
        Ok(false)
      } else {
        Err(Error::FilesystemIo {
          path: path.into(),
          source,
        })
      }
    }
  }
}
