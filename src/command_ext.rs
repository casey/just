use super::*;

pub(crate) trait CommandExt {
  fn export(
    &mut self,
    settings: &Settings,
    dotenv: &BTreeMap<String, String>,
    scope: &Scope,
    unsets: &HashSet<String>,
  );

  fn export_scope(&mut self, settings: &Settings, scope: &Scope, unsets: &HashSet<String>);
}

impl CommandExt for Command {
  fn export(
    &mut self,
    settings: &Settings,
    dotenv: &BTreeMap<String, String>,
    scope: &Scope,
    unsets: &HashSet<String>,
  ) {
    for (name, value) in dotenv {
      self.env(name, value);
    }

    if let Some(parent) = scope.parent() {
      self.export_scope(settings, parent, unsets);
    }
  }

  fn export_scope(&mut self, settings: &Settings, scope: &Scope, unsets: &HashSet<String>) {
    if let Some(parent) = scope.parent() {
      self.export_scope(settings, parent, unsets);
    }

    for unset in unsets {
      self.env_remove(unset);
    }

    for binding in scope.bindings() {
      if settings.export || binding.export {
        self.env(binding.name.lexeme(), &binding.value);
      }
    }
  }
}
