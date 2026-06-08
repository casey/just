use super::*;

pub(crate) enum Resolution<'src> {
  Disabled(BTreeSet<Modulepath>),
  Resolved(Arc<Recipe<'src>>),
}
