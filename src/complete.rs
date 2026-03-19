use {super::*, clap::builder::StyledStr};

pub(crate) fn argument(current: &OsStr) -> Vec<CompletionCandidate> {
  let Some(current) = current.to_str() else {
    return Vec::new();
  };

  let loader = Loader::new();

  let context = match Context::new(&loader) {
    Ok(context) => context,
    Err(_) => return Vec::new(),
  };

  let mut candidates = context.candidate_recipes(&current);

  for (name, binding) in &context.justfile.assignments {
    if !binding.private && name.starts_with(current) {
      candidates.push(CompletionCandidate::new(format!("{name}=")))
    }
  }

  candidates
}

pub(crate) fn recipe(current: &OsStr) -> Vec<CompletionCandidate> {
  let Some(current) = current.to_str() else {
    return Vec::new();
  };

  let loader = Loader::new();

  let context = match Context::new(&loader) {
    Ok(context) => context,
    Err(_) => return Vec::new(),
  };

  context.candidate_recipes(&current)
}

pub(crate) fn variable(current: &OsStr) -> Vec<CompletionCandidate> {
  let Some(current) = current.to_str() else {
    return Vec::new();
  };

  let loader = Loader::new();

  let context = match Context::new(&loader) {
    Ok(context) => context,
    Err(_) => return Vec::new(),
  };

  context
    .justfile
    .assignments
    .into_iter()
    .filter(|(name, binding)| !binding.private && name.starts_with(current))
    .map(|(name, _)| CompletionCandidate::new(name))
    .collect()
}

struct Context<'src> {
  config: Config,
  justfile: Justfile<'src>,
}

impl<'src> Context<'src> {
  fn candidate_recipes(&self, current: &str) -> Vec<CompletionCandidate> {
    let mut candidates = Vec::new();

    for recipe in self.justfile.public_recipes_recursive(&self.config) {
      let path = recipe.recipe_path().to_string();

      if path.starts_with(current) {
        candidates
          .push(CompletionCandidate::new(path).help(recipe.doc.as_ref().map(StyledStr::from)));
      }
    }

    candidates
  }

  fn new(loader: &'src Loader) -> RunResult<'src, Self> {
    let matches = Config::app()
      .ignore_errors(true)
      .try_get_matches_from(env::args_os())
      .map_err(|err| Error::internal(format!("failed to parse arguments: {err}")))?;

    let config = Config::from_matches(&matches)?;

    let search = Search::search(&config)?;

    let compilation = Compiler::compile(&config, &loader, &search.justfile)?;

    Ok(Context {
      config,
      justfile: compilation.justfile,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn script_generation() {
    #[track_caller]
    fn case(shell: Shell) {
      assert!(!shell.completion_script().is_empty());
    }

    case(Shell::Bash);
    case(Shell::Elvish);
    case(Shell::Fish);
    case(Shell::Powershell);
    case(Shell::Zsh);
    case(Shell::Nushell);
  }

  #[test]
  fn bash_script_contains_just_complete() {
    assert!(Shell::Bash.completion_script().contains("JUST_COMPLETE"));
  }
}
