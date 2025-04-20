use super::{Ast, Name};

#[derive(Debug, Clone)]
pub struct ModuleInline<'src> {
  pub(crate) ast: Ast<'src>,
  pub(crate) doc: Option<String>,
  pub(crate) groups: Vec<String>,
  pub(crate) name: Name<'src>,
}
