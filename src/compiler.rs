use crate::common::*;

pub(crate) struct Compiler;

impl Compiler {
  pub(crate) fn compile(text: &str) -> CompilationResult<Justfile> {
    let tokens = Lexer::lex(text)?;

    let ast = Parser::parse(&tokens)?;

    Analyzer::analyze(ast)
  }
}
