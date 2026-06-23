use super::*;

pub(crate) enum Resolution<T> {
  Disabled(BTreeSet<Modulepath>),
  Resolved(T),
}

impl<'src> Resolution<Arc<Recipe<'src>>> {
  pub(crate) fn resolve<'a>(
    path: &Namepath<'src>,
    mut modules: &'a Table<'src, Justfile<'src>>,
    mut absent_modules: &'a BTreeSet<String>,
    mut recipes: &'a Table<'src, Arc<Recipe<'src>>>,
    mut disabled_recipes: &'a Table<'src, Disabled<'src>>,
  ) -> Option<Self> {
    let (name, prefix) = path.split_last();

    let mut walked = Vec::new();

    for component in prefix {
      let lexeme = component.lexeme();
      walked.push(lexeme.to_string());

      if let Some(module) = modules.get(lexeme) {
        modules = &module.modules;
        absent_modules = &module.absent_modules;
        recipes = &module.recipes;
        disabled_recipes = &module.disabled_recipes;
      } else if absent_modules.contains(lexeme) {
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
      disabled_recipes
        .get(name.lexeme())
        .map(|disabled| Self::Disabled(disabled.modules.clone()))
    }
  }
}
