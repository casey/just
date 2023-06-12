use super::*;

#[derive(Debug)]
pub(crate) enum Error<'src> {
  ArgumentCountMismatch {
    recipe: &'src str,
    parameters: Vec<Parameter<'src>>,
    found: usize,
    min: usize,
    max: usize,
  },
  Backtick {
    token: Token<'src>,
    output_error: OutputError,
  },
  ChooserInvoke {
    shell_binary: String,
    shell_arguments: String,
    chooser: OsString,
    io_error: io::Error,
  },
  ChooserRead {
    chooser: OsString,
    io_error: io::Error,
  },
  ChooserStatus {
    chooser: OsString,
    status: ExitStatus,
  },
  ChooserWrite {
    chooser: OsString,
    io_error: io::Error,
  },
  CircularInclude {
    current: PathBuf,
    include: PathBuf,
  },
  Code {
    recipe: &'src str,
    line_number: Option<usize>,
    code: i32,
    print_message: bool,
  },
  CommandInvoke {
    binary: OsString,
    arguments: Vec<OsString>,
    io_error: io::Error,
  },
  CommandStatus {
    binary: OsString,
    arguments: Vec<OsString>,
    status: ExitStatus,
  },
  Compile {
    compile_error: CompileError<'src>,
  },
  Config {
    config_error: ConfigError,
  },
  Cygpath {
    recipe: &'src str,
    output_error: OutputError,
  },
  DefaultRecipeRequiresArguments {
    recipe: &'src str,
    min_arguments: usize,
  },
  Dotenv {
    dotenv_error: dotenvy::Error,
  },
  DumpJson {
    serde_json_error: serde_json::Error,
  },
  EditorInvoke {
    editor: OsString,
    io_error: io::Error,
  },
  EditorStatus {
    editor: OsString,
    status: ExitStatus,
  },
  EvalUnknownVariable {
    variable: String,
    suggestion: Option<Suggestion<'src>>,
  },
  FormatCheckFoundDiff,
  FunctionCall {
    function: Name<'src>,
    message: String,
  },
  IncludeMissingPath {
    file: PathBuf,
    line: usize,
  },
  InitExists {
    justfile: PathBuf,
  },
  Internal {
    message: String,
  },
  InvalidDirective {
    line: String,
  },
  Io {
    recipe: &'src str,
    io_error: io::Error,
  },
  Load {
    path: PathBuf,
    io_error: io::Error,
  },
  NoChoosableRecipes,
  NoRecipes,
  RegexCompile {
    source: regex::Error,
  },
  Search {
    search_error: SearchError,
  },
  Shebang {
    recipe: &'src str,
    command: String,
    argument: Option<String>,
    io_error: io::Error,
  },
  Signal {
    recipe: &'src str,
    line_number: Option<usize>,
    signal: i32,
  },
  TmpdirIo {
    recipe: &'src str,
    io_error: io::Error,
  },
  Unknown {
    recipe: &'src str,
    line_number: Option<usize>,
  },
  UnknownOverrides {
    overrides: Vec<String>,
  },
  UnknownRecipes {
    recipes: Vec<String>,
    suggestion: Option<Suggestion<'src>>,
  },
  Unstable {
    message: String,
  },
  WriteJustfile {
    justfile: PathBuf,
    io_error: io::Error,
  },
}

impl<'src> Error<'src> {
  pub(crate) fn code(&self) -> Option<i32> {
    match self {
      Self::Code { code, .. }
      | Self::Backtick {
        output_error: OutputError::Code(code),
        ..
      } => Some(*code),
      Self::ChooserStatus { status, .. } | Self::EditorStatus { status, .. } => status.code(),
      _ => None,
    }
  }

  fn context(&self) -> Option<Token<'src>> {
    match self {
      Self::Backtick { token, .. } => Some(*token),
      Self::Compile { compile_error } => Some(compile_error.context()),
      Self::FunctionCall { function, .. } => Some(function.token()),
      _ => None,
    }
  }

  pub(crate) fn internal(message: impl Into<String>) -> Self {
    Self::Internal {
      message: message.into(),
    }
  }

  pub(crate) fn print_message(&self) -> bool {
    !matches!(
      self,
      Error::Code {
        print_message: false,
        ..
      }
    )
  }
}

impl<'src> From<CompileError<'src>> for Error<'src> {
  fn from(compile_error: CompileError<'src>) -> Self {
    Self::Compile { compile_error }
  }
}

impl<'src> From<ConfigError> for Error<'src> {
  fn from(config_error: ConfigError) -> Self {
    Self::Config { config_error }
  }
}

impl<'src> From<dotenvy::Error> for Error<'src> {
  fn from(dotenv_error: dotenvy::Error) -> Error<'src> {
    Self::Dotenv { dotenv_error }
  }
}

impl<'src> From<SearchError> for Error<'src> {
  fn from(search_error: SearchError) -> Self {
    Self::Search { search_error }
  }
}

impl<'src> ColorDisplay for Error<'src> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    use Error::*;

    write!(
      f,
      "{}: {}",
      color.error().paint("error"),
      color.message().prefix()
    )?;

    match self {
      ArgumentCountMismatch {
        recipe,
        found,
        min,
        max,
        ..
      } => {
        if min == max {
          let expected = min;
          write!(
            f,
            "Recipe `{recipe}` got {found} {} but {}takes {expected}",
            Count("argument", *found),
            if expected < found { "only " } else { "" }
          )?;
        } else if found < min {
          write!(
            f,
            "Recipe `{recipe}` got {found} {} but takes at least {min}",
            Count("argument", *found)
          )?;
        } else if found > max {
          write!(
            f,
            "Recipe `{recipe}` got {found} {} but takes at most {max}",
            Count("argument", *found)
          )?;
        }
      }
      Backtick { output_error, .. } => match output_error {
        OutputError::Code(code) => {
          write!(f, "Backtick failed with exit code {code}")?;
        }
        OutputError::Signal(signal) => {
          write!(f, "Backtick was terminated by signal {signal}")?;
        }
        OutputError::Unknown => {
          write!(f, "Backtick failed for an unknown reason")?;
        }
        OutputError::Io(io_error) => {
          match io_error.kind() {
            io::ErrorKind::NotFound => write!(
              f,
              "Backtick could not be run because just could not find the shell:\n{io_error}"
            ),
            io::ErrorKind::PermissionDenied => write!(
              f,
              "Backtick could not be run because just could not run the shell:\n{io_error}"
            ),
            _ => write!(
              f,
              "Backtick could not be run because of an IO error while launching the shell:\n{io_error}"
            ),
          }?;
        }
        OutputError::Utf8(utf8_error) => {
          write!(
            f,
            "Backtick succeeded but stdout was not utf8: {utf8_error}"
          )?;
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
          "Chooser `{shell_binary} {shell_arguments} {}` invocation failed: {io_error}",
          chooser.to_string_lossy(),
        )?;
      }
      ChooserRead { chooser, io_error } => {
        write!(
          f,
          "Failed to read output from chooser `{}`: {io_error}",
          chooser.to_string_lossy()
        )?;
      }
      ChooserStatus { chooser, status } => {
        write!(
          f,
          "Chooser `{}` failed: {status}",
          chooser.to_string_lossy()
        )?;
      }
      ChooserWrite { chooser, io_error } => {
        write!(
          f,
          "Failed to write to chooser `{}`: {io_error}",
          chooser.to_string_lossy()
        )?;
      }
      CircularInclude { current, include } => {
        write!(
          f,
          "Include `{}` in `{}` is a circular include", include.display(), current.display()
        )?;
      },
      Code {
        recipe,
        line_number,
        code,
        ..
      } => {
        if let Some(n) = line_number {
          write!(
            f,
            "Recipe `{recipe}` failed on line {n} with exit code {code}"
          )?;
        } else {
          write!(f, "Recipe `{recipe}` failed with exit code {code}")?;
        }
      }
      CommandInvoke {
        binary,
        arguments,
        io_error,
      } => {
        write!(
          f,
          "Failed to invoke {}: {io_error}",
          iter::once(binary)
            .chain(arguments)
            .map(|value| Enclosure::tick(value.to_string_lossy()).to_string())
            .collect::<Vec<String>>()
            .join(" "),
        )?;
      }
      CommandStatus {
        binary,
        arguments,
        status,
      } => {
        write!(
          f,
          "Command {} failed: {status}",
          iter::once(binary)
            .chain(arguments)
            .map(|value| Enclosure::tick(value.to_string_lossy()).to_string())
            .collect::<Vec<String>>()
            .join(" "),
        )?;
      }
      Compile { compile_error } => Display::fmt(compile_error, f)?,
      Config { config_error } => Display::fmt(config_error, f)?,
      Cygpath {
        recipe,
        output_error,
      } => match output_error {
        OutputError::Code(code) => {
          write!(
            f,
            "Cygpath failed with exit code {code} while translating recipe `{recipe}` shebang interpreter \
             path"
          )?;
        }
        OutputError::Signal(signal) => {
          write!(
            f,
            "Cygpath terminated by signal {signal} while translating recipe `{recipe}` shebang interpreter \
             path"
          )?;
        }
        OutputError::Unknown => {
          write!(
            f,
            "Cygpath experienced an unknown failure while translating recipe `{recipe}` shebang \
             interpreter path"
          )?;
        }
        OutputError::Io(io_error) => {
          match io_error.kind() {
            io::ErrorKind::NotFound => write!(
              f,
              "Could not find `cygpath` executable to translate recipe `{recipe}` shebang interpreter \
               path:\n{io_error}"
            ),
            io::ErrorKind::PermissionDenied => write!(
              f,
              "Could not run `cygpath` executable to translate recipe `{recipe}` shebang interpreter \
               path:\n{io_error}"
            ),
            _ => write!(f, "Could not run `cygpath` executable:\n{io_error}"),
          }?;
        }
        OutputError::Utf8(utf8_error) => {
          write!(
            f,
            "Cygpath successfully translated recipe `{recipe}` shebang interpreter path, but output was \
             not utf8: {utf8_error}"
          )?;
        }
      },
      DefaultRecipeRequiresArguments {
        recipe,
        min_arguments,
      } => {
        write!(
          f,
          "Recipe `{recipe}` cannot be used as default recipe since it requires at least {min_arguments} {}.",
          Count("argument", *min_arguments),
        )?;
      }
      Dotenv { dotenv_error } => {
        write!(f, "Failed to load environment file: {dotenv_error}")?;
      }
      DumpJson { serde_json_error } => {
        write!(f, "Failed to dump JSON to stdout: {serde_json_error}")?;
      }
      EditorInvoke { editor, io_error } => {
        write!(
          f,
          "Editor `{}` invocation failed: {io_error}",
          editor.to_string_lossy(),
        )?;
      }
      EditorStatus { editor, status } => {
        write!(f, "Editor `{}` failed: {status}", editor.to_string_lossy(),)?;
      }
      EvalUnknownVariable {
        variable,
        suggestion,
      } => {
        write!(f, "Justfile does not contain variable `{variable}`.")?;
        if let Some(suggestion) = *suggestion {
          write!(f, "\n{suggestion}")?;
        }
      }
      FormatCheckFoundDiff => {
        write!(f, "Formatted justfile differs from original.")?;
      }
      FunctionCall { function, message } => {
        write!(
          f,
          "Call to function `{}` failed: {message}",
          function.lexeme()
        )?;
      }
      IncludeMissingPath {
        file: justfile, line
      } => {

        write!(
          f,
          "!include directive on line {} of `{}` has no argument",
          line.ordinal(),
          justfile.display(),
        )?;

      },
      InitExists { justfile } => {
        write!(f, "Justfile `{}` already exists", justfile.display())?;
      }
      Internal { message } => {
        write!(
          f,
          "Internal runtime error, this may indicate a bug in just: {message} \
           consider filing an issue: https://github.com/casey/just/issues/new"
        )?;
      }
      InvalidDirective { line } => {
        write!(f, "Invalid directive: {line}")?;
      }
      Io { recipe, io_error } => {
        match io_error.kind() {
          io::ErrorKind::NotFound => write!(
            f,
            "Recipe `{recipe}` could not be run because just could not find the shell: {io_error}"
          ),
          io::ErrorKind::PermissionDenied => write!(
            f,
            "Recipe `{recipe}` could not be run because just could not run the shell: {io_error}"
          ),
          _ => write!(
            f,
            "Recipe `{recipe}` could not be run because of an IO error while launching the shell: {io_error}"
          ),
        }?;
      }
      Load { io_error, path } => {
        write!(
          f,
          "Failed to read justfile at `{}`: {io_error}",
          path.display()
        )?;
      }
      NoChoosableRecipes => {
        write!(f, "Justfile contains no choosable recipes.")?;
      }
      NoRecipes => {
        write!(f, "Justfile contains no recipes.")?;
      }
      RegexCompile { source } => {
        write!(f, "{source}")?;
      }
      Search { search_error } => Display::fmt(search_error, f)?,
      Shebang {
        recipe,
        command,
        argument,
        io_error,
      } => {
        if let Some(argument) = argument {
          write!(
            f,
            "Recipe `{recipe}` with shebang `#!{command} {argument}` execution error: {io_error}",
          )?;
        } else {
          write!(
            f,
            "Recipe `{recipe}` with shebang `#!{command}` execution error: {io_error}",
          )?;
        }
      }
      Signal {
        recipe,
        line_number,
        signal,
      } => {
        if let Some(n) = line_number {
          write!(
            f,
            "Recipe `{recipe}` was terminated on line {n} by signal {signal}",
          )?;
        } else {
          write!(f, "Recipe `{recipe}` was terminated by signal {signal}")?;
        }
      }
      TmpdirIo { recipe, io_error } => write!(
        f,
        "Recipe `{recipe}` could not be run because of an IO error while trying to create a temporary \
         directory or write a file to that directory`:{io_error}",
      )?,
      Unknown {
        recipe,
        line_number,
      } => {
        if let Some(n) = line_number {
          write!(
            f,
            "Recipe `{recipe}` failed on line {n} for an unknown reason",
          )?;
        } else {
          write!(f, "Recipe `{recipe}` failed for an unknown reason")?;
        }
      }
      UnknownOverrides { overrides } => {
        write!(
          f,
          "{} {} overridden on the command line but not present in justfile",
          Count("Variable", overrides.len()),
          List::and_ticked(overrides),
        )?;
      }
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
          write!(f, "\n{suggestion}")?;
        }
      }
      Unstable { message } => {
        write!(
          f,
          "{message} Invoke `just` with the `--unstable` flag to enable unstable features."
        )?;
      }
      WriteJustfile { justfile, io_error } => {
        write!(
          f,
          "Failed to write justfile to `{}`: {io_error}",
          justfile.display()
        )?;
      }
    }

    write!(f, "{}", color.message().suffix())?;

    if let ArgumentCountMismatch {
      recipe, parameters, ..
    } = self
    {
      writeln!(f)?;
      write!(f, "{}:\n    just {recipe}", color.message().paint("usage"))?;
      for param in parameters {
        write!(f, " {}", param.color_display(color))?;
      }
    }

    if let Some(token) = self.context() {
      writeln!(f)?;
      write!(f, "{}", token.color_display(color.error()))?;
    }

    Ok(())
  }
}
