use super::*;

#[derive(Copy, Clone)]
pub(crate) struct ExecutionContext<'src: 'run, 'run> {
  pub(crate) config: &'run Config,
  pub(crate) dotenv: &'run BTreeMap<String, String>,
  pub(crate) module: &'run Justfile<'src>,
  pub(crate) scope: &'run Scope<'src, 'run>,
  pub(crate) search: &'run Search,
}

impl<'src: 'run, 'run> ExecutionContext<'src, 'run> {
  pub(crate) fn working_directory(&self) -> PathBuf {
    let base = if self.module.is_submodule() {
      &self.module.working_directory
    } else {
      &self.search.working_directory
    };

    if let Some(setting) = &self.module.settings.working_directory {
      base.join(setting)
    } else {
      base.into()
    }
  }
}
