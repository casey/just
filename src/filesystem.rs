use super::*;

pub(crate) fn create_dir_all(path: &Utf8Path) -> RunResult<'static> {
  fs::create_dir_all(path).map_err(|source| Error::FilesystemIo {
    path: path.into(),
    source,
  })
}

pub(crate) fn is_file(path: &Utf8Path) -> RunResult<'static, bool> {
  Ok(metadata_opt(path)?.is_some_and(|metadata| metadata.is_file()))
}

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

pub(crate) fn open(path: &Utf8Path) -> RunResult<'static, File> {
  File::open(path).map_err(|source| Error::FilesystemIo {
    path: path.into(),
    source,
  })
}

pub(crate) fn open_opt(path: &Utf8Path) -> RunResult<'static, Option<File>> {
  match File::open(path) {
    Ok(file) => Ok(Some(file)),
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

pub(crate) fn read_dir_opt(path: &Utf8Path) -> RunResult<'static, Option<fs::ReadDir>> {
  match fs::read_dir(path) {
    Ok(read_dir) => Ok(Some(read_dir)),
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

pub(crate) fn remove_file(path: &Utf8Path) -> RunResult<'static> {
  fs::remove_file(path).map_err(|source| Error::FilesystemIo {
    path: path.into(),
    source,
  })
}
