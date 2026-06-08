use super::*;

pub(crate) enum Resolution<'src> {
  Disabled(BTreeSet<Modulepath>),
  Resolved(Arc<Recipe<'src>>),
}

impl<'src> Resolution<'src> {
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
        absent_modules = &module.absent_modules;
        disabled_recipes = &module.disabled_recipes;
        modules = &module.modules;
        recipes = &module.recipes;
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
