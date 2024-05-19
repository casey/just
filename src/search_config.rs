use super::*;

/// Controls how `just` will search for the justfile.
#[derive(Debug, PartialEq)]
pub(crate) enum SearchConfig {
  /// Recursively search for the justfile upwards from the invocation directory
  /// to the root, setting the working directory to the directory in which the
  /// justfile is found.
  FromInvocationDirectory,
  /// As in `Invocation`, but start from `search_directory`.
  FromSearchDirectory { search_directory: PathBuf },
  /// Search for global justfile
  GlobalJustfile,
  /// Use user-specified justfile, with the working directory set to the
  /// directory that contains it.
  WithJustfile { justfile: PathBuf },
  /// Use user-specified justfile and working directory.
  WithJustfileAndWorkingDirectory {
    justfile: PathBuf,
    working_directory: PathBuf,
  },
}
