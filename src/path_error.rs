use super::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)), context(suffix(false)))]
pub(crate) enum PathError {
  #[snafu(display("config directory is not valid unicode: {source}"))]
  ConfigDirectoryUnicode { source: FromPathBufError },
  #[snafu(display("I/O error retrieving current directory: {source}"))]
  CurrentDirectoryIo { source: io::Error },
  #[snafu(display("current directory is not valid unicode: {source}"))]
  CurrentDirectoryUnicode { source: FromPathBufError },
  #[snafu(display("failed to get home directory"))]
  HomeDirectoryMissing,
  #[snafu(display("home directory is not valid unicode: {source}"))]
  HomeDirectoryUnicode { source: FromPathBufError },
  #[snafu(display("runtime directory is not valid unicode: {source}"))]
  RuntimeDirectoryUnicode { source: FromPathBufError },
  #[snafu(display("temporary directory is not valid unicode: `{}`", path.display()))]
  TemporaryDirectoryUnicode { path: std::path::PathBuf },
}
