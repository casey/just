use super::*;

// todo:
// - figure out how to test these
//   - unit tests (will have to inject current_dir and args, and replicate what the completion script passes in)
//   - integration tests (will have to puppet bash, which is annoying)
// - how does clap actually pass arguments to us? do we need to modify env::args_os?

pub(crate) fn argument(current: &OsStr) -> Vec<CompletionCandidate> {
  let loader = Loader::new();

  let Some(context) = Context::new(current, &loader) else {
    return Vec::new();
  };

  let mut candidates = context.candidate_recipes();

  for (name, binding) in &context.justfile.assignments {
    if !binding.private && name.starts_with(context.current) {
      candidates.push(CompletionCandidate::new(format!("{name}=")));
    }
  }

  candidates
}

pub(crate) fn recipe(current: &OsStr) -> Vec<CompletionCandidate> {
  let loader = Loader::new();

  let Some(context) = Context::new(current, &loader) else {
    return Vec::new();
  };

  context.candidate_recipes()
}

pub(crate) fn variable(current: &OsStr) -> Vec<CompletionCandidate> {
  let loader = Loader::new();

  let Some(context) = Context::new(current, &loader) else {
    return Vec::new();
  };

  context
    .justfile
    .assignments
    .into_iter()
    .filter(|(name, binding)| !binding.private && name.starts_with(context.current))
    .map(|(name, _)| CompletionCandidate::new(name))
    .collect()
}

struct Context<'run, 'src> {
  config: Config,
  current: &'run str,
  justfile: Justfile<'src>,
}

impl<'run, 'src> Context<'run, 'src> {
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

  fn try_new(current: &'run str, loader: &'src Loader) -> RunResult<'src, Self> {
    let matches = Arguments::command()
      .ignore_errors(true)
      .try_get_matches_from(env::args_os())
      .map_err(|err| Error::internal(format!("failed to parse arguments: {err}")))?;

    let arguments = Arguments::from_arg_matches(&matches).unwrap();

    let config = Config::from_arguments(arguments)?;

    let search = Search::search(&config)?;

    let compilation = Compiler::compile(&config, loader, &search.justfile)?;

    Ok(Context {
      config,
      current,
      justfile: compilation.justfile,
    })
  }
}
