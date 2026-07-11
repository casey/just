use super::*;

#[derive(Debug, Clone)]
pub(crate) enum Setting<'src> {
  AllowDuplicateRecipes(bool),
  AllowDuplicateVariables(bool),
  DefaultList(bool),
  DefaultScript(bool),
  DotenvCommand(Expression<'src>),
  DotenvFilename(Expression<'src>),
  DotenvLoad(bool),
  DotenvOverride(bool),
  DotenvPath(Expression<'src>),
  DotenvRequired(bool),
  Export(bool),
  Fallback(bool),
  Guards(bool),
  IgnoreComments(bool),
  Indentation(StringLiteral<'src>, Indentation),
  Lazy(bool),
  Lists(bool),
  MinimumVersion(StringLiteral<'src>),
  NoCd(bool),
  NoExitMessage(bool),
  PositionalArguments(bool),
  Quiet(bool),
  ScriptInterpreter(Interpreter<Expression<'src>>),
  Shell(Interpreter<Expression<'src>>),
  Tempdir(Expression<'src>),
  Unstable(bool),
  WindowsPowerShell(bool),
  WindowsShell(Interpreter<Expression<'src>>),
  WorkingDirectory(Expression<'src>),
}

impl<'src> Setting<'src> {
  pub(crate) fn is_default(&self) -> bool {
    match self {
      Self::AllowDuplicateRecipes(value)
      | Self::AllowDuplicateVariables(value)
      | Self::DefaultList(value)
      | Self::DefaultScript(value)
      | Self::DotenvLoad(value)
      | Self::DotenvOverride(value)
      | Self::DotenvRequired(value)
      | Self::Export(value)
      | Self::Fallback(value)
      | Self::Guards(value)
      | Self::IgnoreComments(value)
      | Self::Lazy(value)
      | Self::Lists(value)
      | Self::NoCd(value)
      | Self::NoExitMessage(value)
      | Self::PositionalArguments(value)
      | Self::Quiet(value)
      | Self::Unstable(value)
      | Self::WindowsPowerShell(value) => *value,
      Self::DotenvCommand(_value)
      | Self::DotenvFilename(_value)
      | Self::DotenvPath(_value)
      | Self::Tempdir(_value)
      | Self::WorkingDirectory(_value) => false,
      Self::Indentation(..) | Self::MinimumVersion(_) => false,
      Self::ScriptInterpreter(_value) | Self::Shell(_value) | Self::WindowsShell(_value) => false,
    }
  }

  pub(crate) fn expressions_mut(&mut self) -> impl Iterator<Item = &mut Expression<'src>> {
    let (first, rest) = match self {
      Self::DotenvCommand(value)
      | Self::DotenvFilename(value)
      | Self::DotenvPath(value)
      | Self::Tempdir(value)
      | Self::WorkingDirectory(value) => (Some(value), None),
      Self::ScriptInterpreter(value) | Self::Shell(value) | Self::WindowsShell(value) => (
        Some(&mut value.command),
        Some(value.arguments.as_mut_slice()),
      ),
      _ => (None, None),
    };

    first.into_iter().chain(rest.into_iter().flatten())
  }

  pub(crate) fn conflicts(&self) -> &'static [Keyword] {
    match self {
      Self::DotenvCommand(_) => &[
        Keyword::DotenvFilename,
        Keyword::DotenvLoad,
        Keyword::DotenvPath,
        Keyword::DotenvRequired,
      ],
      Self::DotenvFilename(_)
      | Self::DotenvLoad(true)
      | Self::DotenvPath(_)
      | Self::DotenvRequired(true) => &[Keyword::DotenvCommand],
      Self::NoCd(true) => &[Keyword::WorkingDirectory],
      Self::WorkingDirectory(_) => &[Keyword::NoCd],
      _ => &[],
    }
  }
}

impl Display for Setting<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::AllowDuplicateRecipes(value)
      | Self::AllowDuplicateVariables(value)
      | Self::DefaultList(value)
      | Self::DefaultScript(value)
      | Self::DotenvLoad(value)
      | Self::DotenvOverride(value)
      | Self::DotenvRequired(value)
      | Self::Export(value)
      | Self::Fallback(value)
      | Self::Guards(value)
      | Self::IgnoreComments(value)
      | Self::Lazy(value)
      | Self::Lists(value)
      | Self::NoCd(value)
      | Self::NoExitMessage(value)
      | Self::PositionalArguments(value)
      | Self::Quiet(value)
      | Self::Unstable(value)
      | Self::WindowsPowerShell(value) => write!(f, "{value}"),
      Self::DotenvCommand(value)
      | Self::DotenvFilename(value)
      | Self::DotenvPath(value)
      | Self::Tempdir(value)
      | Self::WorkingDirectory(value) => {
        write!(f, "{value}")
      }
      Self::Indentation(value, _) | Self::MinimumVersion(value) => write!(f, "{value}"),
      Self::ScriptInterpreter(value) | Self::Shell(value) | Self::WindowsShell(value) => {
        write!(f, "[{value}]")
      }
    }
  }
}
