use super::*;

pub(crate) fn config_directory() -> PathResult<Option<Utf8PathBuf>> {
  dirs::config_dir()
    .map(|path| path.into_utf8().context(path_error::ConfigDirectoryUnicode))
    .transpose()
}

pub(crate) fn current_directory() -> PathResult<Utf8PathBuf> {
  env::current_dir()
    .context(path_error::CurrentDirectoryIo)?
    .try_into()
    .context(path_error::CurrentDirectoryUnicode)
}

pub(crate) fn home_directory() -> PathResult<Option<Utf8PathBuf>> {
  dirs::home_dir()
    .map(|path| path.into_utf8().context(path_error::HomeDirectoryUnicode))
    .transpose()
}

pub(crate) fn home_directory_required() -> PathResult<Utf8PathBuf> {
  home_directory()?.context(path_error::HomeDirectoryMissing)
}

pub(crate) fn runtime_directory() -> PathResult<Option<Utf8PathBuf>> {
  dirs::runtime_dir()
    .map(|path| {
      path
        .into_utf8()
        .context(path_error::RuntimeDirectoryUnicode)
    })
    .transpose()
}

pub(crate) fn temporary_directory(tempdir: &TempDir) -> PathResult<&Utf8Path> {
  Utf8Path::from_path(tempdir.path()).context(path_error::TemporaryDirectoryUnicode {
    path: tempdir.path(),
  })
}
