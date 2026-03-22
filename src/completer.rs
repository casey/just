use super::*;

pub(crate) struct Completer<'run, 'src> {
  config: Config,
  current: &'run str,
  justfile: Justfile<'src>,
}

impl<'run, 'src> Completer<'run, 'src> {
  pub(crate) fn argument(current: &OsStr) -> Vec<CompletionCandidate> {
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

    candidates
  }

  fn candidate_recipes(&self) -> Vec<CompletionCandidate> {
    let mut candidates = Vec::new();

    for recipe in self.justfile.public_recipes_recursive(&self.config) {
      let path = recipe.recipe_path().to_string();

      if path.starts_with(self.current) {
        candidates
          .push(CompletionCandidate::new(path).help(recipe.doc.as_ref().map(StyledStr::from)));
      }
    }

    candidates
  }

  fn new(current: &'run OsStr, loader: &'src Loader) -> Option<Self> {
    Self::try_new(current.to_str()?, loader).ok()
  }

  pub(crate) fn recipe(current: &OsStr) -> Vec<CompletionCandidate> {
    let loader = Loader::new();

    let Some(completer) = Completer::new(current, &loader) else {
      return Vec::new();
    };

    completer.candidate_recipes()
  }

  fn try_new(current: &'run str, loader: &'src Loader) -> RunResult<'src, Self> {
    let mut args = env::args_os().collect::<Vec<OsString>>();

    args.drain(1..3);

    let matches = Arguments::command()
      .ignore_errors(true)
      .try_get_matches_from(args)
      .map_err(|err| Error::internal(format!("failed to parse arguments: {err}")))?;

    let arguments = Arguments::from_arg_matches(&matches).unwrap();

    let config = Config::from_arguments(arguments).unwrap_or(Config {
      invocation_directory: env::current_dir().context(config_error::CurrentDir)?,
      ..Config::default()
    });

    let search = Search::search(&config)?;

    let compilation = Compiler::compile(&config, loader, &search.justfile)?;

    Ok(Completer {
      config,
      current,
      justfile: compilation.justfile,
    })
  }

  pub(crate) fn variable(current: &OsStr) -> Vec<CompletionCandidate> {
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
}
