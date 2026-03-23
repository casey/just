use super::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)), context(suffix(false)))]
pub(crate) enum Error {
  #[snafu(display("I/O error at `{}`", path.display()))]
  Io {
    source: std::io::Error,
    path: PathBuf,
  },
  #[snafu(display("failed to parse `{}`", path.display()))]
  ParseWorkflow {
    source: serde_yaml::Error,
    path: PathBuf,
  },
  #[snafu(display("action version mismatch"))]
  Mismatch,
  #[snafu(display("failed to parse version `{version}`"))]
  ParseVersion {
    source: semver::Error,
    version: String,
  },
  #[snafu(display("failed to run `gh` for `{repo}`"))]
  GhInvoke {
    source: std::io::Error,
    repo: String,
  },
  #[snafu(display("`gh` for `{repo}` returned {status}"))]
  GhStatus { repo: String, status: ExitStatus },
  #[snafu(display("`gh` output was not unicode"))]
  GhOutput { source: Utf8Error },
  #[snafu(display("failed to get latest version for `{repo}`"))]
  LatestVersion { repo: String },
  #[snafu(display("failed to parse uses `{uses}`"))]
  UsesParse { uses: String },
}
