use crate::common::*;

pub(crate) enum Error<'src> {
  Search(SearchError),
  Compile(CompilationError<'src>),
  Config(ConfigError),
  // TODO: remove this variant
  Code(i32),
  Run(RuntimeError<'src>),
}

// TODO:
// - errors should have a `Display` method that prints the error message with no
//   newline and no color
// - Error may bold the message outside of the Display impl
// - errors should have a `Context` method
// - Remove Color::fmt(f)

impl<'src> Error<'src> {
  pub(crate) fn code(&self) -> i32 {
    match self {
      Self::Search(_) | Self::Compile(_) | Self::Config(_) => EXIT_FAILURE,
      Self::Code(code) => *code,
      Self::Run(error) => error.code(),
    }
  }

  fn context(&self) -> Option<Token<'src>> {
    match self {
      Self::Search(_) | Self::Config(_) | Self::Code(_) => None,
      Self::Compile(error) => Some(error.context()),
      Self::Run(error) => error.context(),
    }
  }

  pub(crate) fn write(&self, w: &mut dyn Write, color: Color) -> io::Result<()> {
    if let Error::Code(_) = self {
      return Ok(());
    }

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
      token.write_context_2(w, color.error())?;
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
      Self::Code(_) => Ok(()),
      Self::Run(error) => Display::fmt(error, f),
    }
  }
}
