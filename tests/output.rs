use super::*;

pub(crate) struct Output {
  pub(crate) pid: u32,
  pub(crate) stdout: String,
  pub(crate) tempdir: TempDir,
}

impl Output {
  pub(crate) fn test(self) -> Test {
    Test::with_tempdir(self.tempdir)
  }
}
