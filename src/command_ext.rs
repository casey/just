use crate::common::*;

pub(crate) trait CommandExt {
  fn export(&mut self, dotenv: &BTreeMap<String, String>, scope: &Scope);

  fn export_scope(&mut self, scope: &Scope);
}

impl CommandExt for Command {
  fn export(&mut self, dotenv: &BTreeMap<String, String>, scope: &Scope) {
    for (name, value) in dotenv {
      self.env(name, value);
    }

    if let Some(parent) = scope.parent() {
      self.export_scope(parent);
    }
  }

  fn export_scope(&mut self, scope: &Scope) {
    if let Some(parent) = scope.parent() {
      self.export_scope(parent);
    }

    for binding in scope.bindings() {
      if binding.export {
        self.env(binding.name.lexeme(), &binding.value);
      }
    }
  }
}
