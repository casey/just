use crate::common::*;

pub(crate) trait CommandExt {
  fn export(&mut self, settings: &Settings, dotenv: &BTreeMap<String, String>, scope: &Scope);

  fn export_scope(&mut self, settings: &Settings, scope: &Scope);
}

impl CommandExt for Command {
  fn export(&mut self, settings: &Settings, dotenv: &BTreeMap<String, String>, scope: &Scope) {
    for (name, value) in dotenv {
      self.env(name, value);
    }

    if let Some(parent) = scope.parent() {
      self.export_scope(settings, parent);
    }
  }

  fn export_scope(&mut self, settings: &Settings, scope: &Scope) {
    if let Some(parent) = scope.parent() {
      self.export_scope(settings, parent);
    }

    for binding in scope.bindings() {
      if settings.export || (binding.export && binding.condition.unwrap_or(true)) {
        self.env(binding.name.lexeme(), &binding.value);
      }
    }
  }
}
