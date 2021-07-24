use crate::common::*;

#[derive(Debug)]
pub(crate) enum RuntimeError<'src> {
  ArgumentCountMismatch {
    recipe:     &'src str,
    parameters: Vec<Parameter<'src>>,
    found:      usize,
    min:        usize,
    max:        usize,
  },
  Backtick {
    token:        Token<'src>,
    output_error: OutputError,
  },
  Code {
    recipe:      &'src str,
    line_number: Option<usize>,
    code:        i32,
  },
  CommandInvocation {
    binary:    OsString,
    arguments: Vec<OsString>,
    io_error:  io::Error,
  },
  Cygpath {
    recipe:       &'src str,
    output_error: OutputError,
  },
  Dotenv {
    dotenv_error: dotenv::Error,
  },
  EvalUnknownVariable {
    variable:   String,
    suggestion: Option<Suggestion<'src>>,
  },
  FunctionCall {
    function: Name<'src>,
    message:  String,
  },
  Internal {
    message: String,
  },
  Io {
    recipe:   &'src str,
    io_error: io::Error,
  },
  Load {
    path:     PathBuf,
    io_error: io::Error,
  },
  Shebang {
    recipe:   &'src str,
    command:  String,
    argument: Option<String>,
    io_error: io::Error,
  },
  Signal {
    recipe:      &'src str,
    line_number: Option<usize>,
    signal:      i32,
  },
  TmpdirIo {
    recipe:   &'src str,
    io_error: io::Error,
  },
  UnknownOverrides {
    overrides: Vec<String>,
  },
  UnknownRecipes {
    recipes:    Vec<String>,
    suggestion: Option<Suggestion<'src>>,
  },
  Unknown {
    recipe:      &'src str,
    line_number: Option<usize>,
  },
  NoRecipes,
  DefaultRecipeRequiresArguments {
    recipe:        &'src str,
    min_arguments: usize,
  },
}

impl<'src> RuntimeError<'src> {
  pub(crate) fn code(&self) -> i32 {
    match *self {
      Self::Code { code, .. }
      | Self::Backtick {
        output_error: OutputError::Code(code),
        ..
      } => code,
      _ => EXIT_FAILURE,
    }
  }

  fn context(&self) -> Option<Token> {
    use RuntimeError::*;
    match self {
      FunctionCall { function, .. } => Some(function.token()),
      Backtick { token, .. } => Some(*token),
      _ => None,
    }
  }

  pub(crate) fn internal(message: impl Into<String>) -> Self {
    Self::Internal {
      message: message.into(),
    }
  }
}

impl<'src> Display for RuntimeError<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    use RuntimeError::*;

    let color = if f.alternate() {
      Color::always()
    } else {
      Color::never()
    };
    let message = color.message();
    write!(f, "{}", message.prefix())?;

    match self {
      EvalUnknownVariable {
        variable,
        suggestion,
      } => {
        write!(f, "Justfile does not contain variable `{}`.", variable,)?;
        if let Some(suggestion) = *suggestion {
          write!(f, "\n{}", suggestion)?;
        }
      },
      UnknownRecipes {
        recipes,
        suggestion,
      } => {
        write!(
          f,
          "Justfile does not contain {} {}.",
          Count("recipe", recipes.len()),
          List::or_ticked(recipes),
        )?;
        if let Some(suggestion) = *suggestion {
          write!(f, "\n{}", suggestion)?;
        }
      },
      UnknownOverrides { overrides } => {
        write!(
          f,
          "{} {} overridden on the command line but not present in justfile",
          Count("Variable", overrides.len()),
          List::and_ticked(overrides),
        )?;
      },
      ArgumentCountMismatch {
        recipe,
        parameters,
        found,
        min,
        max,
      } => {
        if min == max {
          let expected = min;
          write!(
            f,
            "Recipe `{}` got {} {} but {}takes {}",
            recipe,
            found,
            Count("argument", *found),
            if expected < found { "only " } else { "" },
            expected
          )?;
        } else if found < min {
          write!(
            f,
            "Recipe `{}` got {} {} but takes at least {}",
            recipe,
            found,
            Count("argument", *found),
            min
          )?;
        } else if found > max {
          write!(
            f,
            "Recipe `{}` got {} {} but takes at most {}",
            recipe,
            found,
            Count("argument", *found),
            max
          )?;
        }
        write!(f, "\nusage:\n    just {}", recipe)?;
        for param in parameters {
          if color.stderr().active() {
            write!(f, " {:#}", param)?;
          } else {
            write!(f, " {}", param)?;
          }
        }
      },
      Code {
        recipe,
        line_number,
        code,
      } =>
        if let Some(n) = line_number {
          write!(
            f,
            "Recipe `{}` failed on line {} with exit code {}",
            recipe, n, code
          )?;
        } else {
          write!(f, "Recipe `{}` failed with exit code {}", recipe, code)?;
        },
      CommandInvocation {
        binary,
        arguments,
        io_error,
      } => {
        write!(
          f,
          "Failed to invoke {}: {}",
          iter::once(binary)
            .chain(arguments)
            .map(|value| Enclosure::tick(value.to_string_lossy()).to_string())
            .collect::<Vec<String>>()
            .join(" "),
          io_error,
        )?;
      },
      Cygpath {
        recipe,
        output_error,
      } => match output_error {
        OutputError::Code(code) => {
          write!(
            f,
            "Cygpath failed with exit code {} while translating recipe `{}` shebang interpreter \
             path",
            code, recipe
          )?;
        },
        OutputError::Signal(signal) => {
          write!(
            f,
            "Cygpath terminated by signal {} while translating recipe `{}` shebang interpreter \
             path",
            signal, recipe
          )?;
        },
        OutputError::Unknown => {
          write!(
            f,
            "Cygpath experienced an unknown failure while translating recipe `{}` shebang \
             interpreter path",
            recipe
          )?;
        },
        OutputError::Io(io_error) => {
          match io_error.kind() {
            io::ErrorKind::NotFound => write!(
              f,
              "Could not find `cygpath` executable to translate recipe `{}` shebang interpreter \
               path:\n{}",
              recipe, io_error
            ),
            io::ErrorKind::PermissionDenied => write!(
              f,
              "Could not run `cygpath` executable to translate recipe `{}` shebang interpreter \
               path:\n{}",
              recipe, io_error
            ),
            _ => write!(f, "Could not run `cygpath` executable:\n{}", io_error),
          }?;
        },
        OutputError::Utf8(utf8_error) => {
          write!(
            f,
            "Cygpath successfully translated recipe `{}` shebang interpreter path, but output was \
             not utf8: {}",
            recipe, utf8_error
          )?;
        },
      },
      Dotenv { dotenv_error } => {
        writeln!(f, "Failed to load .env: {}", dotenv_error)?;
      },
      FunctionCall { function, message } => {
        writeln!(
          f,
          "Call to function `{}` failed: {}",
          function.lexeme(),
          message
        )?;
      },
      Shebang {
        recipe,
        command,
        argument,
        io_error,
      } =>
        if let Some(argument) = argument {
          write!(
            f,
            "Recipe `{}` with shebang `#!{} {}` execution error: {}",
            recipe, command, argument, io_error
          )?;
        } else {
          write!(
            f,
            "Recipe `{}` with shebang `#!{}` execution error: {}",
            recipe, command, io_error
          )?;
        },
      Signal {
        recipe,
        line_number,
        signal,
      } =>
        if let Some(n) = line_number {
          write!(
            f,
            "Recipe `{}` was terminated on line {} by signal {}",
            recipe, n, signal
          )?;
        } else {
          write!(f, "Recipe `{}` was terminated by signal {}", recipe, signal)?;
        },
      Unknown {
        recipe,
        line_number,
      } =>
        if let Some(n) = line_number {
          write!(
            f,
            "Recipe `{}` failed on line {} for an unknown reason",
            recipe, n
          )?;
        } else {
          write!(f, "Recipe `{}` failed for an unknown reason", recipe)?;
        },
      Io { recipe, io_error } => {
        match io_error.kind() {
          io::ErrorKind::NotFound => writeln!(
            f,
            "Recipe `{}` could not be run because just could not find `sh`:{}",
            recipe, io_error
          ),
          io::ErrorKind::PermissionDenied => writeln!(
            f,
            "Recipe `{}` could not be run because just could not run `sh`:{}",
            recipe, io_error
          ),
          _ => writeln!(
            f,
            "Recipe `{}` could not be run because of an IO error while launching `sh`:{}",
            recipe, io_error
          ),
        }?;
      },
      Load { io_error, path } => {
        // TODO: test this error message
        writeln!(
          f,
          "Failed to read justffile at `{}`: {}",
          path.display(),
          io_error
        )?;
      },
      TmpdirIo { recipe, io_error } => writeln!(
        f,
        "Recipe `{}` could not be run because of an IO error while trying to create a temporary \
         directory or write a file to that directory`:{}",
        recipe, io_error
      )?,
      Backtick { output_error, .. } => match output_error {
        OutputError::Code(code) => {
          writeln!(f, "Backtick failed with exit code {}", code)?;
        },
        OutputError::Signal(signal) => {
          writeln!(f, "Backtick was terminated by signal {}", signal)?;
        },
        OutputError::Unknown => {
          writeln!(f, "Backtick failed for an unknown reason")?;
        },
        OutputError::Io(io_error) => {
          match io_error.kind() {
            io::ErrorKind::NotFound => write!(
              f,
              "Backtick could not be run because just could not find `sh`:\n{}",
              io_error
            ),
            io::ErrorKind::PermissionDenied => write!(
              f,
              "Backtick could not be run because just could not run `sh`:\n{}",
              io_error
            ),
            _ => write!(
              f,
              "Backtick could not be run because of an IO error while launching `sh`:\n{}",
              io_error
            ),
          }?;
        },
        OutputError::Utf8(utf8_error) => {
          writeln!(
            f,
            "Backtick succeeded but stdout was not utf8: {}",
            utf8_error
          )?;
        },
      },
      NoRecipes => {
        writeln!(f, "Justfile contains no recipes.",)?;
      },
      DefaultRecipeRequiresArguments {
        recipe,
        min_arguments,
      } => {
        writeln!(
          f,
          "Recipe `{}` cannot be used as default recipe since it requires at least {} {}.",
          recipe,
          min_arguments,
          Count("argument", *min_arguments),
        )?;
      },
      Internal { message } => {
        write!(
          f,
          "Internal runtime error, this may indicate a bug in just: {} \
           consider filing an issue: https://github.com/casey/just/issues/new",
          message
        )?;
      },
    }

    write!(f, "{}", message.suffix())?;

    if let Some(token) = self.context() {
      token.write_context(f, Color::fmt(f).error())?;
    }

    Ok(())
  }
}

impl<'src> From<dotenv::Error> for RuntimeError<'src> {
  fn from(dotenv_error: dotenv::Error) -> RuntimeError<'src> {
    RuntimeError::Dotenv { dotenv_error }
  }
}
