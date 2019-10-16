use crate::common::*;

pub(crate) trait CompilationResultExt {
  fn expected(self, kinds: &[TokenKind]) -> Self;
}

impl<'src, T> CompilationResultExt for CompilationResult<'src, T> {
  fn expected(mut self, kinds: &[TokenKind]) -> Self {
    if let Err(CompilationError {
      kind: CompilationErrorKind::UnexpectedToken {
        ref mut expected, ..
      },
      ..
    }) = &mut self
    {
      expected.extend_from_slice(kinds);
      expected.sort();
      expected.dedup();
    }

    self
  }
}
