use super::*;

pub(crate) enum Executor<'a> {
  Command(&'a [StringLiteral<'a>]),
  Shebang(Shebang<'a>),
}

impl<'a> Executor<'a> {
  pub(crate) fn command<'src>(
    &self,
    path: &Path,
    recipe: &'src str,
    working_directory: Option<&Path>,
  ) -> RunResult<'src, Command> {
    match self {
      Self::Command(args) => {
        let mut command = Command::new(&args[0].cooked);

        if let Some(working_directory) = working_directory {
          command.current_dir(working_directory);
        }

        for arg in &args[1..] {
          command.arg(&arg.cooked);
        }

        command.arg(path);

        Ok(command)
      }
      Self::Shebang(shebang) => {
        // make script executable
        Platform::set_execute_permission(path).map_err(|error| Error::TempdirIo {
          recipe,
          io_error: error,
        })?;

        // create command to run script
        Platform::make_shebang_command(path, working_directory, *shebang).map_err(|output_error| {
          Error::Cygpath {
            recipe,
            output_error,
          }
        })
      }
    }
  }

  pub(crate) fn include_first_line(&self) -> bool {
    match self {
      Self::Command(_) => true,
      Self::Shebang(shebang) => shebang.include_shebang_line(),
    }
  }

  pub(crate) fn script_filename(&self, recipe: &str, extension: Option<&str>) -> String {
    match self {
      Self::Command(_) => {
        let mut filename = recipe.to_string();

        if let Some(extension) = extension {
          filename.push_str(extension);
        }

        filename
      }
      Self::Shebang(shebang) => shebang.script_filename(recipe, extension),
    }
  }

  pub(crate) fn error<'src>(&self, io_error: io::Error, recipe: &'src str) -> Error<'src> {
    match self {
      Self::Command(args) => {
        let mut command = String::new();

        for (i, arg) in args.iter().enumerate() {
          if i > 0 {
            command.push(' ');
          }
          command.push_str(&arg.cooked);
        }

        Error::Script {
          command,
          io_error,
          recipe,
        }
      }
      Self::Shebang(shebang) => Error::Shebang {
        argument: shebang.argument.map(String::from),
        command: shebang.interpreter.to_owned(),
        io_error,
        recipe,
      },
    }
  }
}
