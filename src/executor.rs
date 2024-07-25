use super::*;

pub(crate) enum Executor<'a> {
  Command(&'a Interpreter<'a>),
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
      Self::Command(interpreter) => {
        let mut command = Command::new(&interpreter.command.cooked);

        if let Some(working_directory) = working_directory {
          command.current_dir(working_directory);
        }

        for arg in &interpreter.arguments {
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

  pub(crate) fn script_filename(&self, recipe: &str, extension: Option<&str>) -> String {
    let extension = extension.unwrap_or_else(|| {
      let interpreter = match self {
        Self::Command(interpreter) => &interpreter.command.cooked,
        Self::Shebang(shebang) => shebang.interpreter_filename(),
      };

      match interpreter {
        "cmd" | "cmd.exe" => ".bat",
        "powershell" | "powershell.exe" | "pwsh" | "pwsh.exe" => ".ps1",
        _ => "",
      }
    });

    format!("{recipe}{extension}")
  }

  pub(crate) fn error<'src>(&self, io_error: io::Error, recipe: &'src str) -> Error<'src> {
    match self {
      Self::Command(Interpreter { command, arguments }) => {
        let mut command = command.cooked.clone();

        for arg in arguments {
          command.push(' ');
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

  // Script text for `recipe` given evaluated `lines` including blanks so line
  // numbers in errors from generated script match justfile source lines.
  pub(crate) fn script<D>(&self, recipe: &Recipe<D>, lines: &[String]) -> String {
    let mut script = String::new();

    match self {
      Self::Shebang(shebang) => {
        let mut n = 0;

        for (i, (line, evaluated)) in recipe.body.iter().zip(lines).enumerate() {
          if i == 0 {
            if shebang.include_shebang_line() {
              script.push_str(evaluated);
              script.push('\n');
              n += 1;
            }
          } else {
            while n < line.number && !line.is_shebang() {
              script.push('\n');
              n += 1;
            }

            script.push_str(evaluated);
            script.push('\n');
            n += 1;
          }
        }
      }
      Self::Command(_) => {
        let mut n = 0;

        for (line, evaluated) in recipe.body.iter().zip(lines) {
          while n < line.number {
            script.push('\n');
            n += 1;
          }

          script.push_str(evaluated);
          script.push('\n');
          n += 1;
        }
      }
    }

    script
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn shebang_script_filename() {
    #[track_caller]
    fn case(interpreter: &str, recipe: &str, extension: Option<&str>, expected: &str) {
      assert_eq!(
        Executor::Shebang(Shebang::new(&format!("#!{interpreter}")).unwrap())
          .script_filename(recipe, extension),
        expected
      );
      assert_eq!(
        Executor::Command(&Interpreter {
          command: StringLiteral::from_raw(interpreter),
          arguments: Vec::new()
        })
        .script_filename(recipe, extension),
        expected
      );
    }

    case("bar", "foo", Some(".sh"), "foo.sh");
    case("pwsh.exe", "foo", Some(".sh"), "foo.sh");
    case("cmd.exe", "foo", Some(".sh"), "foo.sh");
    case("powershell", "foo", None, "foo.ps1");
    case("pwsh", "foo", None, "foo.ps1");
    case("powershell.exe", "foo", None, "foo.ps1");
    case("pwsh.exe", "foo", None, "foo.ps1");
    case("cmd", "foo", None, "foo.bat");
    case("cmd.exe", "foo", None, "foo.bat");
    case("bar", "foo", None, "foo");
  }
}
