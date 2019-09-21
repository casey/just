use crate::common::*;

pub(crate) trait CommandExt {
  fn export_environment_variables<'a>(
    &mut self,
    scope: &BTreeMap<&'a str, String>,
    dotenv: &BTreeMap<String, String>,
    exports: &BTreeSet<&'a str>,
  ) -> RunResult<'a, ()>;
}

impl CommandExt for Command {
  fn export_environment_variables<'a>(
    &mut self,
    scope: &BTreeMap<&'a str, String>,
    dotenv: &BTreeMap<String, String>,
    exports: &BTreeSet<&'a str>,
  ) -> RunResult<'a, ()> {
    for (name, value) in dotenv {
      self.env(name, value);
    }
    for name in exports {
      if let Some(value) = scope.get(name) {
        self.env(name, value);
      } else {
        return Err(RuntimeError::Internal {
          message: format!("scope does not contain exported variable `{}`", name),
        });
      }
    }
    Ok(())
  }
}
