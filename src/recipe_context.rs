use crate::common::*;

pub(crate) struct RecipeContext<'src: 'run, 'run> {
  pub(crate) config: &'run Config,
  pub(crate) scope: Scope<'src, 'run>,
  pub(crate) search: &'run Search,
  pub(crate) settings: &'run Settings<'src>,
}
