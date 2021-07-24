use crate::common::*;

pub(crate) enum JustError<'src> {
  Search(SearchError),
  Compile(CompilationError<'src>),
  Config(ConfigError),
  Load(LoadError),
  // TODO: remove this variant
  Code(i32),
  Run(RuntimeError<'src>),
}

impl<'src> JustError<'src> {
  pub(crate) fn code(&self) -> i32 {
    match self {
      Self::Search(error) => error.code(),
      Self::Compile(error) => error.code(),
      Self::Config(error) => error.code(),
      Self::Load(error) => error.code(),
      Self::Code(code) => *code,
      Self::Run(error) => error.code(),
    }
  }
}

impl<'src> From<SearchError> for JustError<'src> {
  fn from(error: SearchError) -> Self {
    Self::Search(error)
  }
}

impl<'src> From<CompilationError<'src>> for JustError<'src> {
  fn from(error: CompilationError<'src>) -> Self {
    Self::Compile(error)
  }
}

impl<'src> From<RuntimeError<'src>> for JustError<'src> {
  fn from(error: RuntimeError<'src>) -> Self {
    Self::Run(error)
  }
}

impl<'src> From<LoadError> for JustError<'src> {
  fn from(error: LoadError) -> Self {
    Self::Load(error)
  }
}

impl<'src> Display for JustError<'src> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Search(error) => Display::fmt(error, f),
      Self::Compile(error) => Display::fmt(error, f),
      Self::Config(error) => Display::fmt(error, f),
      Self::Load(error) => Display::fmt(error, f),
      Self::Code(_) => Ok(()),
      Self::Run(error) => Display::fmt(error, f),
    }
  }
}
