use common::*;

pub fn parse_success(text: &str) -> Justfile {
  match Parser::parse(text) {
    Ok(justfile) => justfile,
    Err(error) => panic!("Expected successful parse but got error:\n{}", error),
  }
}

macro_rules! compilation_error_test {
  (
    name:     $name:ident,
    input:    $input:expr,
    index:    $index:expr,
    line:     $line:expr,
    column:   $column:expr,
    width:    $width:expr,
    kind:     $kind:expr,
  ) => {
    #[test]
    fn $name() {
      let input = $input;

      let expected = ::CompilationError {
        text: input,
        index: $index,
        line: $line,
        column: $column,
        width: $width,
        kind: $kind,
      };

      let tokens = ::Lexer::lex(input).unwrap();
      let parser = ::Parser::new(input, tokens);

      if let Err(error) = parser.justfile() {
        assert_eq!(error.text, expected.text);
        assert_eq!(error.index, expected.index);
        assert_eq!(error.line, expected.line);
        assert_eq!(error.column, expected.column);
        assert_eq!(error.width, expected.width);
        assert_eq!(error.kind, expected.kind);
        assert_eq!(error, expected);
      } else {
        panic!("parse succeeded but expected: {}\n{}", expected, input);
      }
    }
  };
}
