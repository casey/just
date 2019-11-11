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
      $crate::testing::analysis_error($input, $offset, $line, $column, $width, $kind);
    }
  };
}

pub(crate) fn analysis_error(
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

macro_rules! run_error {
  {
    name: $name:ident,
    src:  $src:expr,
    args: $args:expr,
    error: $error:pat,
    check: $check:block $(,)?
  } => {
    #[test]
    fn $name() {
      let config = &$crate::testing::config(&$args);
      let current_dir = std::env::current_dir().unwrap();

      if let Subcommand::Run{ overrides, arguments } = &config.subcommand {
        match $crate::compiler::Compiler::compile(&$crate::testing::unindent($src))
          .expect("Expected successful compilation")
          .run(
            config,
            &current_dir,
            &overrides,
            &arguments,
          ).expect_err("Expected runtime error") {
            $error => $check
            other => {
              panic!("Unexpected run error: {:?}", other);
            }
          }
      } else {
          panic!("Unexpected subcommand: {:?}", config.subcommand);
      }
    }
  };
}
