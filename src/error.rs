use crate::common::*;

pub(crate) enum Error<'src> {
  Search(SearchError),
  Compile(CompilationError<'src>),
  Config(ConfigError),
  Run(RuntimeError<'src>),
}

// TODO:
// - errors should have a `Context` method
// - Remove Color::fmt(f)
// - fold runtimeerror into Error?
// - sort error enum variants and match statments

impl<'src> Error<'src> {
  pub(crate) fn code(&self) -> i32 {
    match self {
      Self::Search(_) | Self::Compile(_) | Self::Config(_) => EXIT_FAILURE,
      Self::Run(error) => error.code().unwrap_or(EXIT_FAILURE),
    }
  }

  fn context(&self) -> Option<Token<'src>> {
    match self {
      Self::Search(_) | Self::Config(_) => None,
      Self::Compile(error) => Some(error.context()),
      Self::Run(error) => error.context(),
    }
  }

  pub(crate) fn write(&self, w: &mut dyn Write, color: Color) -> io::Result<()> {
    let color = color.stderr();

    if color.active() {
      writeln!(
        w,
        "{}: {}{:#}{}",
        color.error().paint("error"),
        color.message().prefix(),
        self,
        color.message().suffix()
      )?;
    } else {
      writeln!(w, "error: {}", self)?;
    }

    if let Some(token) = self.context() {
      token.write_context(w, color.error())?;
      writeln!(w)?;
    }

    Ok(())
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
      Self::Run(error) => Display::fmt(error, f),
    }
  }
}
