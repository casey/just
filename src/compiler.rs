use super::*;

pub(crate) struct Compiler;

impl Compiler {
  #[cfg(test)]
  pub(crate) fn compile(src: &str) -> CompileResult<Compilation> {
    let root_ast = Self::parse(src)?;
    let root_justfile = Analyzer::analyze(&root_ast, &[])?;

    Ok(Compilation {
      root_ast,
      root_justfile,
      root_source: src,
    })
  }

  pub(crate) fn parse(src: &str) -> CompileResult<Ast> {
    let tokens = Lexer::lex(src)?;
    Parser::parse(&tokens)
  }
}

/// This type represents everything necessary to perform any operation on a justfile - the raw
/// source, the compiled justfile and ast, and references to any included justfiles.
#[derive(Debug)]
pub(crate) struct Compilation<'src> {
  root_ast: Ast<'src>,
  root_source: &'src str,
  root_justfile: Justfile<'src>,
}

impl<'src> Compilation<'src> {
  pub(crate) fn new(
    root_ast: Ast<'src>,
    root_justfile: Justfile<'src>,
    root_source: &'src str,
  ) -> Self {
    Self {
      root_ast,
      root_justfile,
      root_source,
    }
  }

  pub(crate) fn justfile(&self) -> &Justfile<'src> {
    &self.root_justfile
  }

  #[cfg(test)]
  pub(crate) fn into_justfile(self) -> Justfile<'src> {
    self.root_justfile
  }

  pub(crate) fn ast(&self) -> &Ast<'src> {
    &self.root_ast
  }

  pub(crate) fn src(&self) -> &'src str {
    self.root_source
  }
}
