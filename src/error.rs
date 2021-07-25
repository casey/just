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
  pub(crate) fn code(&self) -> Option<i32> {
    match self {
      Self::Search(_) | Self::Compile(_) | Self::Config(_) => None,
      Self::Run(error) => match error {
        RuntimeError::Code { code, .. }
        | RuntimeError::Backtick {
          output_error: OutputError::Code(code),
          ..
        } => Some(*code),
        RuntimeError::ChooserStatus { status, .. } | RuntimeError::EditorStatus { status, .. } =>
          status.code(),
        _ => None,
      },
    }
  }

  fn context(&self) -> Option<Token<'src>> {
    match self {
      Self::Search(_) | Self::Config(_) => None,
      Self::Compile(error) => Some(error.context()),
      Self::Run(error) => match error {
        RuntimeError::FunctionCall { function, .. } => Some(function.token()),
        RuntimeError::Backtick { token, .. } => Some(*token),
        _ => None,
      },
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
      Self::Run(error) => {
        use RuntimeError::*;

        match error {
          EditorInvoke { editor, io_error } => {
            write!(
              f,
              "Editor `{}` invocation failed: {}",
              editor.to_string_lossy(),
              io_error
            )?;
          },
          EditorStatus { editor, status } => {
            write!(
              f,
              "Editor `{}` failed: {}",
              editor.to_string_lossy(),
              status
            )?;
          },
          EvalUnknownVariable {
            variable,
            suggestion,
          } => {
            write!(f, "Justfile does not contain variable `{}`.", variable,)?;
            if let Some(suggestion) = *suggestion {
              write!(f, "\n{}", suggestion)?;
            }
          },
          ChooserInvoke {
            shell_binary,
            shell_arguments,
            chooser,
            io_error,
          } => {
            write!(
              f,
              "Chooser `{} {} {}` invocation failed: {}",
              shell_binary,
              shell_arguments,
              chooser.to_string_lossy(),
              io_error,
            )?;
          },
          ChooserRead { chooser, io_error } => {
            write!(
              f,
              "Failed to read output from chooser `{}`: {}",
              chooser.to_string_lossy(),
              io_error
            )?;
          },
          ChooserStatus { chooser, status } => {
            write!(
              f,
              "Chooser `{}` failed: {}",
              chooser.to_string_lossy(),
              status
            )?;
          },
          ChooserWrite { chooser, io_error } => {
            write!(
              f,
              "Failed to write to chooser `{}`: {}",
              chooser.to_string_lossy(),
              io_error
            )?;
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
            // TODO:
            // - This usage string shouldn't be bolded
            // - parameters should be colored
            write!(f, "\nusage:\n    just {}", recipe)?;
            for param in parameters {
              write!(f, " {}", param)?;
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
          CommandStatus {
            binary,
            arguments,
            status,
          } => {
            write!(
              f,
              "Command {} failed: {}",
              iter::once(binary)
                .chain(arguments)
                .map(|value| Enclosure::tick(value.to_string_lossy()).to_string())
                .collect::<Vec<String>>()
                .join(" "),
              status,
            )?;
          },
          Cygpath {
            recipe,
            output_error,
          } => match output_error {
            OutputError::Code(code) => {
              write!(
                f,
                "Cygpath failed with exit code {} while translating recipe `{}` shebang \
                 interpreter path",
                code, recipe
              )?;
            },
            OutputError::Signal(signal) => {
              write!(
                f,
                "Cygpath terminated by signal {} while translating recipe `{}` shebang \
                 interpreter path",
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
                  "Could not find `cygpath` executable to translate recipe `{}` shebang \
                   interpreter path:\n{}",
                  recipe, io_error
                ),
                io::ErrorKind::PermissionDenied => write!(
                  f,
                  "Could not run `cygpath` executable to translate recipe `{}` shebang \
                   interpreter path:\n{}",
                  recipe, io_error
                ),
                _ => write!(f, "Could not run `cygpath` executable:\n{}", io_error),
              }?;
            },
            OutputError::Utf8(utf8_error) => {
              write!(
                f,
                "Cygpath successfully translated recipe `{}` shebang interpreter path, but output \
                 was not utf8: {}",
                recipe, utf8_error
              )?;
            },
          },
          Dotenv { dotenv_error } => {
            write!(f, "Failed to load .env: {}", dotenv_error)?;
          },
          FunctionCall { function, message } => {
            write!(
              f,
              "Call to function `{}` failed: {}",
              function.lexeme(),
              message
            )?;
          },
          InitExists { justfile } => {
            write!(f, "Justfile `{}` already exists", justfile.display())?;
          },
          WriteJustfile { justfile, io_error } => {
            write!(
              f,
              "Failed to write justfile to `{}`: {}",
              justfile.display(),
              io_error
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
          Unstable { message } => {
            write!(
              f,
              "{} Invoke `just` with the `--unstable` flag to enable unstable features.",
              message
            )?;
          },
          Io { recipe, io_error } => {
            // TODO:
            // - tests io error spacing
            // - what is the error even? better name?
            match io_error.kind() {
              io::ErrorKind::NotFound => write!(
                f,
                "Recipe `{}` could not be run because just could not find `sh`:{}",
                recipe, io_error
              ),
              io::ErrorKind::PermissionDenied => write!(
                f,
                "Recipe `{}` could not be run because just could not run `sh`:{}",
                recipe, io_error
              ),
              _ => write!(
                f,
                "Recipe `{}` could not be run because of an IO error while launching `sh`:{}",
                recipe, io_error
              ),
            }?;
          },
          Load { io_error, path } => {
            // TODO: test this error message
            write!(
              f,
              "Failed to read justffile at `{}`: {}",
              path.display(),
              io_error
            )?;
          },
          TmpdirIo { recipe, io_error } => write!(
            f,
            "Recipe `{}` could not be run because of an IO error while trying to create a \
             temporary directory or write a file to that directory`:{}",
            recipe, io_error
          )?,
          Backtick { output_error, .. } => match output_error {
            OutputError::Code(code) => {
              write!(f, "Backtick failed with exit code {}", code)?;
            },
            OutputError::Signal(signal) => {
              write!(f, "Backtick was terminated by signal {}", signal)?;
            },
            OutputError::Unknown => {
              write!(f, "Backtick failed for an unknown reason")?;
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
              write!(
                f,
                "Backtick succeeded but stdout was not utf8: {}",
                utf8_error
              )?;
            },
          },
          NoChoosableRecipes => {
            write!(f, "Justfile contains no choosable recipes.")?;
          },
          NoRecipes => {
            write!(f, "Justfile contains no recipes.")?;
          },
          DefaultRecipeRequiresArguments {
            recipe,
            min_arguments,
          } => {
            write!(
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

        Ok(())
      },
    }
  }
}
