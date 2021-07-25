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
  ChooserInvoke {
    shell_binary:    String,
    shell_arguments: String,
    chooser:         OsString,
    io_error:        io::Error,
  },
  ChooserRead {
    chooser:  OsString,
    io_error: io::Error,
  },
  ChooserStatus {
    chooser: OsString,
    status:  ExitStatus,
  },
  ChooserWrite {
    chooser:  OsString,
    io_error: io::Error,
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
  CommandStatus {
    binary:    OsString,
    arguments: Vec<OsString>,
    status:    ExitStatus,
  },
  Cygpath {
    recipe:       &'src str,
    output_error: OutputError,
  },
  DefaultRecipeRequiresArguments {
    recipe:        &'src str,
    min_arguments: usize,
  },
  Dotenv {
    dotenv_error: dotenv::Error,
  },
  EditorInvoke {
    editor:   OsString,
    io_error: io::Error,
  },
  EditorStatus {
    editor: OsString,
    status: ExitStatus,
  },
  EvalUnknownVariable {
    variable:   String,
    suggestion: Option<Suggestion<'src>>,
  },
  FunctionCall {
    function: Name<'src>,
    message:  String,
  },
  InitExists {
    justfile: PathBuf,
  },
  WriteJustfile {
    justfile: PathBuf,
    io_error: io::Error,
  },
  Unstable {
    message: String,
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
  NoRecipes,
  NoChoosableRecipes,
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
  Unknown {
    recipe:      &'src str,
    line_number: Option<usize>,
  },
  UnknownOverrides {
    overrides: Vec<String>,
  },
  UnknownRecipes {
    recipes:    Vec<String>,
    suggestion: Option<Suggestion<'src>>,
  },
}

impl<'src> RuntimeError<'src> {
  pub(crate) fn internal(message: impl Into<String>) -> Self {
    Self::Internal {
      message: message.into(),
    }
  }
}

impl<'src> From<dotenv::Error> for RuntimeError<'src> {
  fn from(dotenv_error: dotenv::Error) -> RuntimeError<'src> {
    RuntimeError::Dotenv { dotenv_error }
  }
}
