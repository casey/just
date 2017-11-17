use common::*;

pub trait CommandExt {
  fn export_environment_variables<'a>(
    &mut self,
    scope: &Map<&'a str, String>,
    exports: &Set<&'a str>
  ) -> Result<(), RuntimeError<'a>>;
}

impl CommandExt for Command {
  fn export_environment_variables<'a>(
    &mut self,
    scope: &Map<&'a str, String>,
    exports: &Set<&'a str>
  ) -> Result<(), RuntimeError<'a>> {
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
