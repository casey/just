use super::*;

pub(crate) enum Resolution<'src> {
  Disabled(BTreeSet<Modulepath>),
  Resolved(Arc<Recipe<'src>>),
}

impl<'src> Resolution<'src> {
  pub(crate) fn resolve<'a>(
    path: &Namepath<'src>,
    mut modules: &'a Table<'src, Justfile<'src>>,
    mut recipes: &'a Table<'src, Arc<Recipe<'src>>>,
    mut disabled: &'a Table<'src, Disabled<'src>>,
    mut absent: &'a BTreeSet<String>,
  ) -> Option<Self> {
    let (name, prefix) = path.split_last();

    let mut walked = Vec::new();

    for component in prefix {
      let lexeme = component.lexeme();
      walked.push(lexeme.to_string());

      if let Some(module) = modules.get(lexeme) {
        absent = &module.absent;
        disabled = &module.disabled;
        modules = &module.modules;
        recipes = &module.recipes;
      } else if absent.contains(lexeme) {
        return Some(Self::Disabled(BTreeSet::from([Modulepath {
          components: walked,
          spaced: false,
        }])));
      } else {
        return None;
      }
    }

    if let Some(recipe) = recipes.get(name.lexeme()) {
      Some(Self::Resolved(Arc::clone(recipe)))
    } else {
      disabled
        .get(name.lexeme())
        .map(|disabled| Self::Disabled(disabled.modules.clone()))
    }
  }
}
