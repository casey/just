use super::*;

pub(crate) struct Compiler;

impl Compiler {
  pub(crate) fn compile(src: &str) -> CompileResult<(Ast, Justfile)> {
    let tokens = Lexer::lex(src)?;
    let ast = Parser::parse(&tokens)?;
    let justfile = Analyzer::analyze(&ast)?;

    Ok((ast, justfile))
  }
}
