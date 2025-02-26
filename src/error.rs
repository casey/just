use super::*;

#[derive(Debug)]
pub(crate) enum Error<'src> {
  AmbiguousModuleFile {
    module: Name<'src>,
    found: Vec<PathBuf>,
  },
  ArgumentCountMismatch {
    recipe: &'src str,
    parameters: Vec<Parameter<'src>>,
    found: usize,
    min: usize,
    max: usize,
  },
  Assert {
    message: String,
  },
  Backtick {
    token: Token<'src>,
    output_error: OutputError,
  },
  RuntimeDirIo {
    io_error: io::Error,
    path: PathBuf,
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
  CircularImport {
    current: PathBuf,
    import: PathBuf,
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
  DotenvRequired,
  DumpJson {
    source: serde_json::Error,
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
  ExcessInvocations {
    invocations: usize,
  },
  ExpectedSubmoduleButFoundRecipe {
    path: String,
  },
  FormatCheckFoundDiff,
  FunctionCall {
    function: Name<'src>,
    message: String,
  },
  GetConfirmation {
    io_error: io::Error,
  },
  Homedir,
  InitExists {
    justfile: PathBuf,
  },
  Internal {
    message: String,
  },
  Io {
    recipe: &'src str,
    io_error: io::Error,
  },
  Load {
    path: PathBuf,
    io_error: io::Error,
  },
  MissingImportFile {
    path: Token<'src>,
  },
  MissingModuleFile {
    module: Name<'src>,
  },
  NoChoosableRecipes,
  NoDefaultRecipe,
  NoRecipes,
  NotConfirmed {
    recipe: &'src str,
  },
  RegexCompile {
    source: regex::Error,
  },
  Script {
    command: String,
    io_error: io::Error,
    recipe: &'src str,
  },
  Search {
    search_error: SearchError,
  },
  Shebang {
    argument: Option<String>,
    command: String,
    io_error: io::Error,
    recipe: &'src str,
  },
  Signal {
    recipe: &'src str,
    line_number: Option<usize>,
    signal: i32,
  },
  StdoutIo {
    io_error: io::Error,
  },
  TempdirIo {
    recipe: &'src str,
    io_error: io::Error,
  },
  TempfileIo {
    io_error: io::Error,
  },
  Unknown {
    recipe: &'src str,
    line_number: Option<usize>,
  },
  UnknownSubmodule {
    path: String,
  },
  UnknownOverrides {
    overrides: Vec<String>,
  },
  UnknownRecipe {
    recipe: String,
    suggestion: Option<Suggestion<'src>>,
  },
  UnstableFeature {
    unstable_feature: UnstableFeature,
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
      Self::AmbiguousModuleFile { module, .. } | Self::MissingModuleFile { module, .. } => {
        Some(module.token)
      }
      Self::Backtick { token, .. } => Some(*token),
      Self::Compile { compile_error } => Some(compile_error.context()),
      Self::FunctionCall { function, .. } => Some(function.token),
      Self::MissingImportFile { path } => Some(*path),
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

impl From<ConfigError> for Error<'_> {
  fn from(config_error: ConfigError) -> Self {
    Self::Config { config_error }
  }
}

impl<'src> From<dotenvy::Error> for Error<'src> {
  fn from(dotenv_error: dotenvy::Error) -> Error<'src> {
    Self::Dotenv { dotenv_error }
  }
}

impl From<SearchError> for Error<'_> {
  fn from(search_error: SearchError) -> Self {
    Self::Search { search_error }
  }
}

impl ColorDisplay for Error<'_> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    use Error::*;

    let error = color.error().paint("error");
    let message = color.message().prefix();
    write!(f, "{error}: {message}")?;

    match self {
      AmbiguousModuleFile { module, found } =>
        write!(f,
          "Found multiple source files for module `{module}`: {}",
          List::and_ticked(found.iter().map(|path| path.display())),
        )?,
      ArgumentCountMismatch { recipe, found, min, max, .. } => {
        let count = Count("argument", *found);
        if min == max {
          let expected = min;
          let only = if expected < found { "only " } else { "" };
          write!(f, "Recipe `{recipe}` got {found} {count} but {only}takes {expected}")?;
        } else if found < min {
          write!(f, "Recipe `{recipe}` got {found} {count} but takes at least {min}")?;
        } else if found > max {
          write!(f, "Recipe `{recipe}` got {found} {count} but takes at most {max}")?;
        }
      }
      Assert { message }=> {
        write!(f, "Assert failed: {message}")?;
      }
      Backtick { output_error, .. } => match output_error {
        OutputError::Code(code) => write!(f, "Backtick failed with exit code {code}")?,
        OutputError::Signal(signal) => write!(f, "Backtick was terminated by signal {signal}")?,
        OutputError::Unknown => write!(f, "Backtick failed for an unknown reason")?,
        OutputError::Io(io_error) => match io_error.kind() {
            io::ErrorKind::NotFound => write!(f, "Backtick could not be run because just could not find the shell:\n{io_error}"),
            io::ErrorKind::PermissionDenied => write!(f, "Backtick could not be run because just could not run the shell:\n{io_error}"),
            _ => write!(f, "Backtick could not be run because of an IO error while launching the shell:\n{io_error}"),
          }?,
        OutputError::Utf8(utf8_error) => write!(f, "Backtick succeeded but stdout was not utf8: {utf8_error}")?,
      }
      ChooserInvoke { shell_binary, shell_arguments, chooser, io_error} => {
        let chooser = chooser.to_string_lossy();
        write!(f, "Chooser `{shell_binary} {shell_arguments} {chooser}` invocation failed: {io_error}")?;
      }
      ChooserRead { chooser, io_error } => {
        let chooser = chooser.to_string_lossy();
        write!(f, "Failed to read output from chooser `{chooser}`: {io_error}")?;
      }
      ChooserStatus { chooser, status } => {
        let chooser = chooser.to_string_lossy();
        write!(f, "Chooser `{chooser}` failed: {status}")?;
      }
      ChooserWrite { chooser, io_error } => {
        let chooser = chooser.to_string_lossy();
        write!(f, "Failed to write to chooser `{chooser}`: {io_error}")?;
      }
      CircularImport { current, import } => {
        let import = import.display();
        let current = current.display();
        write!(f, "Import `{import}` in `{current}` is circular")?;
      }
      Code { recipe, line_number, code, .. } => {
        if let Some(n) = line_number {
          write!(f, "Recipe `{recipe}` failed on line {n} with exit code {code}")?;
        } else {
          write!(f, "Recipe `{recipe}` failed with exit code {code}")?;
        }
      }
      CommandInvoke { binary, arguments, io_error } => {
        let cmd = format_cmd(binary, arguments);
        write!(f, "Failed to invoke {cmd}: {io_error}")?;
      }
      CommandStatus { binary, arguments, status} => {
        let cmd = format_cmd(binary, arguments);
        write!(f, "Command {cmd} failed: {status}")?;
      }
      Compile { compile_error } => Display::fmt(compile_error, f)?,
      Config { config_error } => Display::fmt(config_error, f)?,
      Cygpath { recipe, output_error} => match output_error {
        OutputError::Code(code) => write!(f, "Cygpath failed with exit code {code} while translating recipe `{recipe}` shebang interpreter path")?,
        OutputError::Signal(signal) => write!(f, "Cygpath terminated by signal {signal} while translating recipe `{recipe}` shebang interpreter path")?,
        OutputError::Unknown => write!(f, "Cygpath experienced an unknown failure while translating recipe `{recipe}` shebang interpreter path")?,
        OutputError::Io(io_error) => {
          match io_error.kind() {
            io::ErrorKind::NotFound => write!(f, "Could not find `cygpath` executable to translate recipe `{recipe}` shebang interpreter path:\n{io_error}"),
            io::ErrorKind::PermissionDenied => write!(f, "Could not run `cygpath` executable to translate recipe `{recipe}` shebang interpreter path:\n{io_error}"),
            _ => write!(f, "Could not run `cygpath` executable:\n{io_error}"),
          }?;
        }
        OutputError::Utf8(utf8_error) => write!(f, "Cygpath successfully translated recipe `{recipe}` shebang interpreter path, but output was not utf8: {utf8_error}")?,
      }
      DefaultRecipeRequiresArguments { recipe, min_arguments} => {
        let count = Count("argument", *min_arguments);
        write!(f, "Recipe `{recipe}` cannot be used as default recipe since it requires at least {min_arguments} {count}.")?;
      }
      Dotenv { dotenv_error } => {
        write!(f, "Failed to load environment file: {dotenv_error}")?;
      }
      DotenvRequired => {
        write!(f, "Dotenv file not found")?;
      }
      DumpJson { source } => {
        write!(f, "Failed to dump JSON to stdout: {source}")?;
      }
      EditorInvoke { editor, io_error } => {
        let editor = editor.to_string_lossy();
        write!(f, "Editor `{editor}` invocation failed: {io_error}")?;
      }
      EditorStatus { editor, status } => {
        let editor = editor.to_string_lossy();
        write!(f, "Editor `{editor}` failed: {status}")?;
      }
      EvalUnknownVariable { variable, suggestion} => {
        write!(f, "Justfile does not contain variable `{variable}`.")?;
        if let Some(suggestion) = suggestion {
          write!(f, "\n{suggestion}")?;
        }
      }
      ExcessInvocations { invocations } => {
        write!(f, "Expected 1 command-line recipe invocation but found {invocations}.")?;
      },
      ExpectedSubmoduleButFoundRecipe { path } => {
        write!(f, "Expected submodule at `{path}` but found recipe.")?;
      },
      FormatCheckFoundDiff => {
        write!(f, "Formatted justfile differs from original.")?;
      }
      FunctionCall { function, message } => {
        let function = function.lexeme();
        write!(f, "Call to function `{function}` failed: {message}")?;
      }
      GetConfirmation { io_error } => {
        write!(f, "Failed to read confirmation from stdin: {io_error}")?;
      }
      Homedir => {
        write!(f, "Failed to get homedir")?;
      }
      InitExists { justfile } => {
        write!(f, "Justfile `{}` already exists", justfile.display())?;
      }
      Internal { message } => {
        write!(f, "Internal runtime error, this may indicate a bug in just: {message} \
                   consider filing an issue: https://github.com/casey/just/issues/new")?;
      }
      Io { recipe, io_error } => {
        match io_error.kind() {
          io::ErrorKind::NotFound => write!(f, "Recipe `{recipe}` could not be run because just could not find the shell: {io_error}"),
          io::ErrorKind::PermissionDenied => write!(f, "Recipe `{recipe}` could not be run because just could not run the shell: {io_error}"),
          _ => write!(f, "Recipe `{recipe}` could not be run because of an IO error while launching the shell: {io_error}"),
        }?;
      }
      Load { io_error, path } => {
        write!(f, "Failed to read justfile at `{}`: {io_error}", path.display())?;
      }
      MissingImportFile { .. } => write!(f, "Could not find source file for import.")?,
      MissingModuleFile { module } => write!(f, "Could not find source file for module `{module}`.")?,
      NoChoosableRecipes => write!(f, "Justfile contains no choosable recipes.")?,
      NoDefaultRecipe => write!(f, "Justfile contains no default recipe.")?,
      NoRecipes => write!(f, "Justfile contains no recipes.")?,
      NotConfirmed { recipe } => {
        write!(f, "Recipe `{recipe}` was not confirmed")?;
      }
      RegexCompile { source } => write!(f, "{source}")?,
      RuntimeDirIo { io_error, path } => {
        write!(f, "I/O error in runtime dir `{}`: {io_error}", path.display())?;
      }
      Script { command, io_error, recipe } => {
        write!(f, "Recipe `{recipe}` with command `{command}` execution error: {io_error}")?;
      }
      Search { search_error } => Display::fmt(search_error, f)?,
      Shebang { recipe, command, argument, io_error} => {
        if let Some(argument) = argument {
          write!(f, "Recipe `{recipe}` with shebang `#!{command} {argument}` execution error: {io_error}")?;
        } else {
          write!(f, "Recipe `{recipe}` with shebang `#!{command}` execution error: {io_error}")?;
        }
      }
      Signal { recipe, line_number, signal } => {
        if let Some(n) = line_number {
          write!(f, "Recipe `{recipe}` was terminated on line {n} by signal {signal}")?;
        } else {
          write!(f, "Recipe `{recipe}` was terminated by signal {signal}")?;
        }
      }
      StdoutIo { io_error } => {
        write!(f, "I/O error writing to stdout: {io_error}?")?;
      }
      TempdirIo { recipe, io_error } => {
        write!(f, "Recipe `{recipe}` could not be run because of an IO error while trying to create a temporary \
                   directory or write a file to that directory: {io_error}")?;
      }
      TempfileIo { io_error } => {
        write!(f, "Tempfile I/O error: {io_error}")?;
      }
      Unknown { recipe, line_number} => {
        if let Some(n) = line_number {
          write!(f, "Recipe `{recipe}` failed on line {n} for an unknown reason")?;
        } else {
          write!(f, "Recipe `{recipe}` failed for an unknown reason")?;
        }
      }
      UnknownSubmodule { path } => {
        write!(f, "Justfile does not contain submodule `{path}`")?;
      }
      UnknownOverrides { overrides } => {
        let count = Count("Variable", overrides.len());
        let overrides = List::and_ticked(overrides);
        write!(f, "{count} {overrides} overridden on the command line but not present in justfile")?;
      }
      UnknownRecipe { recipe, suggestion } => {
        write!(f, "Justfile does not contain recipe `{recipe}`")?;
        if let Some(suggestion) = suggestion {
          write!(f, "\n{suggestion}")?;
        }
      }
      UnstableFeature { unstable_feature } => {
        write!(f, "{unstable_feature} Invoke `just` with `--unstable`, set the `JUST_UNSTABLE` environment variable, or add `set unstable` to your `justfile` to enable unstable features.")?;
      }
      WriteJustfile { justfile, io_error } => {
        let justfile = justfile.display();
        write!(f, "Failed to write justfile to `{justfile}`: {io_error}")?;
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

pub(crate) fn render_error(error: &Error, color: Color) {
  match error {
    Error::Compile { compile_error } => compile_error::render_compile_error(compile_error, color),
    _ => eprintln!("{}", error.color_display(color.stderr())),
  }
}

fn format_cmd(binary: &OsString, arguments: &Vec<OsString>) -> String {
  iter::once(binary)
    .chain(arguments)
    .map(|value| Enclosure::tick(value.to_string_lossy()).to_string())
    .collect::<Vec<String>>()
    .join(" ")
}
