use crate::common::*;

pub(crate) enum Error<'src> {
  Search(SearchError),
  Compile(CompilationError<'src>),
  Config(ConfigError),
  // TODO: remove this variant
  Code(i32),
  Run(RuntimeError<'src>),
}

impl<'src> Error<'src> {
  pub(crate) fn code(&self) -> i32 {
    match self {
      Self::Search(_) | Self::Compile(_) | Self::Config(_) => EXIT_FAILURE,
      Self::Code(code) => *code,
      Self::Run(error) => error.code(),
    }
  }

  pub(crate) fn eprint(&self, color: Color) {
    if let Error::Code(_) = self {
      return;
    }

    if color.stderr().active() {
      eprintln!("{}: {:#}", color.stderr().error().paint("error"), self);
    } else {
      eprintln!("error: {}", self);
    }
  }
}

impl<'src> From<SearchError> for Error<'src> {
  fn from(error: SearchError) -> Self {
    Self::Search(error)
  }
}

impl<'src> From<CompilationError<'src>> for Error<'src> {
  fn from(error: CompilationError<'src>) -> Self {
    Self::Compile(error)
  }
}

impl<'src> From<RuntimeError<'src>> for Error<'src> {
  fn from(error: RuntimeError<'src>) -> Self {
    Self::Run(error)
  }
}

impl<'src> Display for Error<'src> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Search(error) => Display::fmt(error, f),
      Self::Compile(error) => Display::fmt(error, f),
      Self::Config(error) => Display::fmt(error, f),
      Self::Code(_) => Ok(()),
      Self::Run(error) => Display::fmt(error, f),
    }
  }
}
