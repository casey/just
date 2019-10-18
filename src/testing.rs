use crate::common::*;

pub(crate) fn parse(text: &str) -> Justfile {
  match Parser::parse(text) {
    Ok(justfile) => justfile,
    Err(error) => panic!("Expected successful parse but got error:\n {}", error),
  }
}

pub(crate) use test_utilities::{tempdir, unindent};

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

      match Parser::parse(text) {
        Ok(_) => panic!("Compilation succeeded but expected: {}\n{}", expected, text),
        Err(actual) => {
          use pretty_assertions::assert_eq;
          assert_eq!(actual, expected);
        }
      }
    }
  };
}
