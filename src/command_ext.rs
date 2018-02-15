use common::*;

pub trait CommandExt {
  fn export_environment_variables<'a>(
    &mut self,
    scope:   &Map<&'a str, String>,
    dotenv:  &Map<String, String>,
    exports: &Set<&'a str>
  ) -> RunResult<'a, ()>;
}

impl CommandExt for Command {
  fn export_environment_variables<'a>(
    &mut self,
    scope:   &Map<&'a str, String>,
    dotenv:  &Map<String, String>,
    exports: &Set<&'a str>
  ) -> RunResult<'a, ()> {
    for (name, value) in dotenv {
      self.env(name, value);
    }
    for name in exports {
      if let Some(value) = scope.get(name) {
        self.env(name, value);
      } else {
        return Err(RuntimeError::Internal {
          message: format!("scope does not contain exported variable `{}`",  name),
        });
      }
    }
    Ok(())
  }
}
