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
  pub(crate) fn tempdir<D>(&self, recipe: &Recipe<'src, D>) -> RunResult<'src, TempDir> {
    let mut tempdir_builder = tempfile::Builder::new();

    tempdir_builder.prefix("just-");

    if let Some(tempdir) = &self.config.tempdir {
      tempdir_builder.tempdir_in(self.search.working_directory.join(tempdir))
    } else {
      match &self.module.settings.tempdir {
        Some(tempdir) => tempdir_builder.tempdir_in(self.search.working_directory.join(tempdir)),
        None => {
          if let Some(runtime_dir) = dirs::runtime_dir() {
            let path = runtime_dir.join("just");
            fs::create_dir_all(&path).map_err(|io_error| Error::RuntimeDirIo {
              io_error,
              path: path.clone(),
            })?;
            tempdir_builder.tempdir_in(path)
          } else {
            tempdir_builder.tempdir()
          }
        }
      }
    }
    .map_err(|error| Error::TempdirIo {
      recipe: recipe.name(),
      io_error: error,
    })
  }

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
