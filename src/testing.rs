use crate::common::*;

pub(crate) fn parse(text: &str) -> Justfile {
  match Analyzer::parse(text) {
    Ok(justfile) => justfile,
    Err(error) => panic!("Expected successful parse but got error:\n {}", error),
  }
}

pub(crate) fn tempdir() -> tempfile::TempDir {
  tempfile::Builder::new()
    .prefix("just-test-tempdir")
    .tempdir()
    .expect("failed to create temporary directory")
}

macro_rules! error_test {
  (
    name:   $name:ident,
    input:  $input:expr,
    offset: $offset:expr,
    line:   $line:expr,
    column: $column:expr,
    width:  $width:expr,
    kind:   $kind:expr,
  ) => {
    #[test]
    fn $name() {
      let text: &str = $input;
      let offset: usize = $offset;
      let column: usize = $column;
      let width: usize = $width;
      let line: usize = $line;
      let kind: CompilationErrorKind = $kind;

      let expected = CompilationError {
        text,
        offset,
        line,
        column,
        width,
        kind,
      };

      match Analyzer::parse(text) {
        Ok(_) => panic!("Compilation succeeded but expected: {}\n{}", expected, text),
        Err(actual) => {
          use pretty_assertions::assert_eq;
          assert_eq!(actual, expected);
        }
      }
    }
  };
}
