use crate::common::*;

pub(crate) trait CommandExt {
  fn export_environment_variables<'a>(
    &mut self,
    scope: &BTreeMap<&'a str, (bool, String)>,
    dotenv: &BTreeMap<String, String>,
  ) -> RunResult<'a, ()>;
}

impl CommandExt for Command {
  fn export_environment_variables<'a>(
    &mut self,
    scope: &BTreeMap<&'a str, (bool, String)>,
    dotenv: &BTreeMap<String, String>,
  ) -> RunResult<'a, ()> {
    for (name, value) in dotenv {
      self.env(name, value);
    }

    for (name, (export, value)) in scope {
      if *export {
        self.env(name, value);
      }
    }

    Ok(())
  }
}
