use crate::common::*;

pub(crate) fn compile(text: &str) -> Justfile {
  match Compiler::compile(text) {
    Ok(justfile) => justfile,
    Err(error) => panic!("Expected successful compilation but got error:\n {}", error),
  }
}

pub(crate) fn config(args: &[&str]) -> Config {
  let mut args = Vec::from(args);
  args.insert(0, "just");

  let app = Config::app();

  let matches = app.get_matches_from_safe(args).unwrap();

  Config::from_matches(&matches).unwrap()
}

pub(crate) use test_utilities::{tempdir, unindent};

macro_rules! analysis_error {
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
      $crate::testing::error($input, $offset, $line, $column, $width, $kind);
    }
  };
}

pub(crate) fn error(
  src: &str,
  offset: usize,
  line: usize,
  column: usize,
  width: usize,
  kind: CompilationErrorKind,
) {
  let expected = CompilationError {
    src,
    offset,
    line,
    column,
    width,
    kind,
  };

  let tokens = Lexer::lex(src).expect("Lexing failed in parse test...");

  let module = Parser::parse(&tokens).expect("Parsing failed in analysis test...");

  match Analyzer::analyze(module) {
    Ok(_) => panic!("Analysis succeeded but expected: {}\n{}", expected, src),
    Err(actual) => {
      assert_eq!(actual, expected);
    }
  }
}

#[test]
fn readme_test() {
  let mut justfiles = vec![];
  let mut current = None;

  for line in fs::read_to_string("README.adoc").unwrap().lines() {
    if let Some(mut justfile) = current {
      if line == "```" {
        justfiles.push(justfile);
        current = None;
      } else {
        justfile += line;
        justfile += "\n";
        current = Some(justfile);
      }
    } else if line == "```make" {
      current = Some(String::new());
    }
  }

  for justfile in justfiles {
    compile(&justfile);
  }
}
