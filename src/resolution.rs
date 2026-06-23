use super::*;

pub(crate) enum Resolution<T> {
  Disabled(BTreeSet<Modulepath>),
  Resolved(T),
}

impl<'src> Resolution<Arc<Recipe<'src>>> {
  pub(crate) fn resolve_recipe<'a>(
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

impl<'src> Resolution<Modulepath> {
  pub(crate) fn resolve_module<'a>(
    target: &Namepath<'src>,
    mut modules: &'a Table<'src, Justfile<'src>>,
    mut absent: &'a BTreeSet<String>,
  ) -> Option<Self> {
    let (last, prefix) = target.split_last();

    let mut walked = Vec::new();

    for component in prefix {
      let module = modules.get(component.lexeme())?;
      modules = &module.modules;
      absent = &module.absent_modules;
      walked.push(component.lexeme().to_string());
    }

    let lexeme = last.lexeme();

    if let Some(module) = modules.get(lexeme) {
      Some(Self::Resolved(module.module_path.clone()))
    } else if absent.contains(lexeme) {
      walked.push(lexeme.to_string());
      Some(Self::Disabled(BTreeSet::from([Modulepath {
        components: walked,
        spaced: false,
      }])))
    } else {
      None
    }
  }
}
