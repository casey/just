use crate::common::*;

use crate::compiler::Compiler;
use pretty_assertions::assert_eq;

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

pub(crate) fn search(config: &Config) -> Search {
  let working_directory = config.invocation_directory.clone();
  let justfile = working_directory.join(crate::search::FILENAME);

  Search {
    justfile,
    working_directory,
  }
}

pub(crate) fn tempdir() -> tempfile::TempDir {
  tempfile::Builder::new()
    .prefix("just-test-tempdir")
    .tempdir()
    .expect("failed to create temporary directory")
}

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
  length: usize,
  kind: CompilationErrorKind,
) {
  let tokens = Lexer::lex(src).expect("Lexing failed in parse test...");

  let ast = Parser::parse(&tokens).expect("Parsing failed in analysis test...");

  match Analyzer::analyze(ast) {
    Ok(_) => panic!("Analysis unexpectedly succeeded"),
    Err(have) => {
      let want = CompilationError {
        token: Token {
          kind: have.token.kind,
          src,
          offset,
          line,
          column,
          length,
        },
        kind,
      };
      assert_eq!(have, want);
    },
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
      let config = $crate::testing::config(&$args);
      let search = $crate::testing::search(&config);

      if let Subcommand::Run{ overrides, arguments } = &config.subcommand {
        match $crate::compiler::Compiler::compile(&$crate::unindent::unindent($src))
          .expect("Expected successful compilation")
          .run(
            &config,
            &search,
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

macro_rules! assert_matches {
  ($expression:expr, $( $pattern:pat )|+ $( if $guard:expr )?) => {
    match $expression {
      $( $pattern )|+ $( if $guard )? => {}
      left => panic!(
        "assertion failed: (left ~= right)\n  left: `{:?}`\n right: `{}`",
        left,
        stringify!($($pattern)|+ $(if $guard)?)
      ),
    }
  }
}
