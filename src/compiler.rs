use super::*;

pub(crate) struct Compiler;

impl Compiler {
  pub(crate) fn parse(src: &str) -> CompileResult<Ast> {
    let tokens = Lexer::lex(src)?;
    Parser::parse(&tokens)
  }
}

/// Wrapper type for an `Ast<'src>` + metadata about where it was parsed from
#[derive(Debug)]
pub(crate) struct AstImport<'src> {
  pub(crate) ast: Ast<'src>,
  //Not currently being used, but could allow for better error messages later
  #[allow(dead_code)]
  import: Import,
}

impl<'src> AstImport<'src> {
  pub(crate) fn new(ast: Ast<'src>, import: Import) -> Self {
    Self { ast, import }
  }
}

/// This type represents everything necessary to perform any operation on a justfile - the raw
/// source, the compiled justfile and ast, and references to any included justfiles.
#[derive(Debug)]
pub(crate) struct Compilation<'src> {
  root_ast: Ast<'src>,
  root_source: &'src str,
  //Not currently being used, but could allow for better error messages later
  #[allow(dead_code)]
  pub(crate) imported_asts: Vec<AstImport<'src>>,
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
      imported_asts: vec![],
    }
  }

  pub(crate) fn with_imports(self, imported_asts: Vec<AstImport<'src>>) -> Self {
    Self {
      imported_asts,
      ..self
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
