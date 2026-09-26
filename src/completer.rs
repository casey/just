use super::*;

pub(crate) struct Completer<'run, 'src> {
  config: Config,
  current: &'run str,
  justfile: Justfile<'src>,
}

impl<'run, 'src> Completer<'run, 'src> {
  fn candidate(&self, name: String, doc: Option<&String>) -> Option<CompletionCandidate> {
    name
      .starts_with(self.current)
      .then(|| CompletionCandidate::new(name).help(doc.map(Into::into)))
  }

  fn candidate_modules(&self) -> Vec<CompletionCandidate> {
    let mut candidates = Vec::new();

    for module in self.justfile.public_modules_recursive(&self.config) {
      let path = module.module_path.to_string();

      if path.starts_with(self.current) {
        candidates.push(CompletionCandidate::new(path).help(module.doc.as_ref().map(Into::into)));
      }
    }

    if self.config.complete_aliases {
      for (alias, modulepath) in self.justfile.public_module_aliases_recursive(&self.config) {
        let name = modulepath.join(alias.name.lexeme()).to_string();
        if name.starts_with(self.current) {
          candidates.push(
            CompletionCandidate::new(name).help(
              self
                .justfile
                .submodule(&alias.target)
                .and_then(|module| module.doc.as_ref())
                .map(Into::into),
            ),
          );
        }
      }
    }

    candidates
  }

  fn candidate_recipes(&self) -> Vec<CompletionCandidate> {
    let mut candidates = Vec::new();

    for recipe in self.justfile.public_recipes_recursive(&self.config) {
      let path = recipe.recipe_path().to_string();

      if path.starts_with(self.current) {
        candidates.push(CompletionCandidate::new(path).help(recipe.doc.as_ref().map(Into::into)));
      }
    }

    if self.config.complete_aliases {
      for (alias, modulepath) in self.justfile.public_aliases_recursive(&self.config) {
        let name = modulepath.join(alias.name.lexeme()).to_string();
        if name.starts_with(self.current) {
          candidates
            .push(CompletionCandidate::new(name).help(alias.target.doc.as_ref().map(Into::into)));
        }
      }
    }

    candidates
  }

  fn candidate_recipes_and_modules(&self) -> Vec<CompletionCandidate> {
    let mut candidates = Vec::new();

    for module in
      iter::once(&self.justfile).chain(self.justfile.public_modules_recursive(&self.config))
    {
      if module.name.is_some()
        && let Some(candidate) = self.candidate(module.module_path.to_string(), module.doc.as_ref())
      {
        candidates.push(candidate);
      }

      candidates.extend(
        module
          .public_recipes(&self.config)
          .into_iter()
          .filter_map(|recipe| {
            self.candidate(recipe.recipe_path().to_string(), recipe.doc.as_ref())
          }),
      );

      if self.config.complete_aliases {
        candidates.extend(
          module
            .recipe_aliases
            .values()
            .filter(|alias| alias.is_public())
            .filter_map(|alias| {
              self.candidate(
                module.module_path.join(alias.name.lexeme()).to_string(),
                alias.target.doc.as_ref(),
              )
            }),
        );

        candidates.extend(
          module
            .module_aliases
            .values()
            .filter(|alias| alias.is_public())
            .filter_map(|alias| {
              self.candidate(
                module.module_path.join(alias.name.lexeme()).to_string(),
                self
                  .justfile
                  .submodule(&alias.target)
                  .and_then(|module| module.doc.as_ref()),
              )
            }),
        );
      }
    }

    candidates
  }

  pub(crate) fn complete_argument(current: &OsStr) -> Vec<CompletionCandidate> {
    let loader = Loader::new();

    let Some(completer) = Completer::new(current, &loader) else {
      return Vec::new();
    };

    let mut candidates = completer.candidate_recipes();

    for (name, binding) in &completer.justfile.assignments {
      if !binding.private && name.starts_with(completer.current) {
        candidates.push(CompletionCandidate::new(format!("{name}=")));
      }
    }

    candidates.extend(PathCompleter::any().complete(current));

    candidates
  }

  pub(crate) fn complete_group(current: &OsStr) -> Vec<CompletionCandidate> {
    let loader = Loader::new();

    let Some(completer) = Completer::new(current, &loader) else {
      return Vec::new();
    };

    completer
      .justfile
      .public_groups(&completer.config)
      .into_iter()
      .filter(|group| group.starts_with(completer.current))
      .map(CompletionCandidate::new)
      .collect()
  }

  pub(crate) fn complete_module(current: &OsStr) -> Vec<CompletionCandidate> {
    let loader = Loader::new();

    let Some(completer) = Completer::new(current, &loader) else {
      return Vec::new();
    };

    completer.candidate_modules()
  }

  pub(crate) fn complete_recipe(current: &OsStr) -> Vec<CompletionCandidate> {
    let loader = Loader::new();

    let Some(completer) = Completer::new(current, &loader) else {
      return Vec::new();
    };

    completer.candidate_recipes()
  }

  pub(crate) fn complete_recipe_or_module(current: &OsStr) -> Vec<CompletionCandidate> {
    let loader = Loader::new();

    let Some(completer) = Completer::new(current, &loader) else {
      return Vec::new();
    };

    completer.candidate_recipes_and_modules()
  }

  pub(crate) fn complete_variable(current: &OsStr) -> Vec<CompletionCandidate> {
    let loader = Loader::new();

    let Some(completer) = Completer::new(current, &loader) else {
      return Vec::new();
    };

    completer
      .justfile
      .assignments
      .into_iter()
      .filter(|(name, binding)| !binding.private && name.starts_with(completer.current))
      .map(|(name, _)| CompletionCandidate::new(name))
      .collect()
  }

  fn config() -> Option<Config> {
    let mut args = env::args_os().collect::<Vec<OsString>>();

    args.drain(1..3);

    let index = env::var("_CLAP_COMPLETE_INDEX")
      .ok()
      .and_then(|index| index.parse::<usize>().ok())
      .unwrap_or(args.len() - 1);

    if (1..args.len()).contains(&index) {
      args.remove(index);
    }

    let matches = Arguments::command()
      .ignore_errors(true)
      .try_get_matches_from(args)
      .ok()?;

    let arguments = Arguments::from_arg_matches(&matches).ok()?;

    Config::from_arguments(arguments).ok()
  }

  fn new(current: &'run OsStr, loader: &'src Loader) -> Option<Self> {
    Self::try_new(current.to_str()?, loader).ok()
  }

  fn try_new(current: &'run str, loader: &'src Loader) -> RunResult<'src, Self> {
    let config = if let Some(config) = Self::config() {
      config
    } else {
      Config::new()?
    };

    let search = Search::search(&config)?;

    let compilation = Compiler::compile(&config, loader, &search.justfile)?;

    Ok(Completer {
      config,
      current,
      justfile: compilation.justfile,
    })
  }
}
