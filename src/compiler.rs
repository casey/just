use crate::common::*;

pub(crate) struct Compiler;

impl Compiler {
  pub(crate) fn compile(src: &str) -> CompilationResult<Justfile> {
    let tokens = Lexer::lex(src)?;

    let ast = Parser::parse(&tokens)?;

    Analyzer::analyze(ast)
  }
}
