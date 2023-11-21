use super::*;

#[derive(Debug)]
pub(crate) struct Compilation<'src> {
  pub(crate) asts: HashMap<PathBuf, Ast<'src>>,
  pub(crate) justfile: Justfile<'src>,
  pub(crate) root: PathBuf,
  pub(crate) srcs: HashMap<PathBuf, &'src str>,
}

impl<'src> Compilation<'src> {
  pub(crate) fn root_ast(&self) -> &Ast<'src> {
    self.asts.get(&self.root).unwrap()
  }

  pub(crate) fn root_src(&self) -> &'src str {
    self.srcs.get(&self.root).unwrap()
  }
}
