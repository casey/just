use super::*;

#[derive(Serialize)]
#[serde(transparent)]
pub(crate) struct Environment {
  pub(crate) variables: BTreeMap<String, Option<String>>,
}

impl Environment {
  pub(crate) fn new(
    dotenv: &BTreeMap<String, String>,
    scope: &Scope,
    settings: &Settings,
    unexports: &HashSet<String>,
  ) -> Environment {
    let mut environment = Self {
      variables: BTreeMap::new(),
    };

    for (name, value) in dotenv {
      environment
        .variables
        .insert(name.clone(), Some(value.clone()));
    }

    if let Some(parent) = scope.parent() {
      environment.scope(parent, settings, unexports);
    }

    environment
  }

  fn scope(&mut self, scope: &Scope, settings: &Settings, unexports: &HashSet<String>) {
    if let Some(parent) = scope.parent() {
      self.scope(parent, settings, unexports);
    }

    for unexport in unexports {
      self.variables.insert(unexport.clone(), None);
    }

    for binding in scope.bindings() {
      if (binding.export || (settings.export && !binding.prelude)) && !binding.value.is_empty() {
        self.variables.insert(
          binding.name.lexeme().to_string(),
          Some(binding.value.join()),
        );
      }
    }
  }

  pub(crate) fn export(&self, command: &mut Command) {
    for (name, value) in &self.variables {
      match value {
        Some(value) => command.env(name, value),
        None => command.env_remove(name),
      };
    }
  }
}
