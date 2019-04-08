use crate::common::*;

use std::process::{Command, ExitStatus, Stdio};

use crate::platform::{Platform, PlatformInterface};

/// Return a `RuntimeError::Signal` if the process was terminated by a signal,
/// otherwise return an `RuntimeError::UnknownFailure`
fn error_from_signal(
  recipe: &str,
  line_number: Option<usize>,
  exit_status: ExitStatus,
) -> RuntimeError {
  match Platform::signal_from_exit_status(exit_status) {
    Some(signal) => RuntimeError::Signal {
      recipe,
      line_number,
      signal,
    },
    None => RuntimeError::Unknown {
      recipe,
      line_number,
    },
  }
}

#[derive(PartialEq, Debug)]
pub struct Recipe<'a> {
  pub dependencies: Vec<&'a str>,
  pub dependency_tokens: Vec<Token<'a>>,
  pub doc: Option<&'a str>,
  pub line_number: usize,
  pub lines: Vec<Vec<Fragment<'a>>>,
  pub name: &'a str,
  pub parameters: Vec<Parameter<'a>>,
  pub private: bool,
  pub quiet: bool,
  pub shebang: bool,
}

pub struct RecipeContext<'a> {
  pub invocation_directory: &'a Result<PathBuf, String>,
  pub configuration: &'a Configuration<'a>,
  pub scope: Map<&'a str, String>,
}

impl<'a> Recipe<'a> {
  pub fn argument_range(&self) -> RangeInclusive<usize> {
    self.min_arguments()..=self.max_arguments()
  }

  pub fn min_arguments(&self) -> usize {
    self
      .parameters
      .iter()
      .filter(|p| p.default.is_none())
      .count()
  }

  pub fn max_arguments(&self) -> usize {
    if self.parameters.iter().any(|p| p.variadic) {
      usize::MAX - 1
    } else {
      self.parameters.len()
    }
  }

  pub fn run(
    &self,
    context: &RecipeContext<'a>,
    arguments: &[&'a str],
    dotenv: &Map<String, String>,
    exports: &Set<&'a str>,
  ) -> RunResult<'a, ()> {
    let configuration = &context.configuration;

    if configuration.verbosity.loquacious() {
      let color = configuration.color.stderr().banner();
      eprintln!(
        "{}===> Running recipe `{}`...{}",
        color.prefix(),
        self.name,
        color.suffix()
      );
    }

    let mut argument_map = Map::new();

    let mut rest = arguments;
    for parameter in &self.parameters {
      let value = if rest.is_empty() {
        match parameter.default {
          Some(ref default) => Cow::Borrowed(default.as_str()),
          None => {
            return Err(RuntimeError::Internal {
              message: "missing parameter without default".to_string(),
            });
          }
        }
      } else if parameter.variadic {
        let value = Cow::Owned(rest.to_vec().join(" "));
        rest = &[];
        value
      } else {
        let value = Cow::Borrowed(rest[0]);
        rest = &rest[1..];
        value
      };
      argument_map.insert(parameter.name, value);
    }

    let mut evaluator = AssignmentEvaluator {
      assignments: &empty(),
      dry_run: configuration.dry_run,
      evaluated: empty(),
      invocation_directory: context.invocation_directory,
      overrides: &empty(),
      quiet: configuration.quiet,
      scope: &context.scope,
      shell: configuration.shell,
      dotenv,
      exports,
    };

    if self.shebang {
      let mut evaluated_lines = vec![];
      for line in &self.lines {
        evaluated_lines.push(evaluator.evaluate_line(line, &argument_map)?);
      }

      if configuration.dry_run || self.quiet {
        for line in &evaluated_lines {
          eprintln!("{}", line);
        }
      }

      if configuration.dry_run {
        return Ok(());
      }

      let tmp = TempDir::new("just").map_err(|error| RuntimeError::TmpdirIoError {
        recipe: self.name,
        io_error: error,
      })?;
      let mut path = tmp.path().to_path_buf();
      path.push(self.name);
      {
        let mut f = fs::File::create(&path).map_err(|error| RuntimeError::TmpdirIoError {
          recipe: self.name,
          io_error: error,
        })?;
        let mut text = String::new();
        // add the shebang
        text += &evaluated_lines[0];
        text += "\n";
        // add blank lines so that lines in the generated script
        // have the same line number as the corresponding lines
        // in the justfile
        for _ in 1..(self.line_number + 2) {
          text += "\n"
        }
        for line in &evaluated_lines[1..] {
          text += line;
          text += "\n";
        }

        if configuration.verbosity.grandiloquent() {
          eprintln!("{}", configuration.color.doc().stderr().paint(&text));
        }

        f.write_all(text.as_bytes())
          .map_err(|error| RuntimeError::TmpdirIoError {
            recipe: self.name,
            io_error: error,
          })?;
      }

      // make the script executable
      Platform::set_execute_permission(&path).map_err(|error| RuntimeError::TmpdirIoError {
        recipe: self.name,
        io_error: error,
      })?;

      let shebang_line = evaluated_lines
        .first()
        .ok_or_else(|| RuntimeError::Internal {
          message: "evaluated_lines was empty".to_string(),
        })?;

      let Shebang {
        interpreter,
        argument,
      } = Shebang::new(shebang_line).ok_or_else(|| RuntimeError::Internal {
        message: format!("bad shebang line: {}", shebang_line),
      })?;

      // create a command to run the script
      let mut command =
        Platform::make_shebang_command(&path, interpreter, argument).map_err(|output_error| {
          RuntimeError::Cygpath {
            recipe: self.name,
            output_error,
          }
        })?;

      command.export_environment_variables(&context.scope, dotenv, exports)?;

      // run it!
      match InterruptHandler::guard(|| command.status()) {
        Ok(exit_status) => {
          if let Some(code) = exit_status.code() {
            if code != 0 {
              return Err(RuntimeError::Code {
                recipe: self.name,
                line_number: None,
                code,
              });
            }
          } else {
            return Err(error_from_signal(self.name, None, exit_status));
          }
        }
        Err(io_error) => {
          return Err(RuntimeError::Shebang {
            recipe: self.name,
            command: interpreter.to_string(),
            argument: argument.map(String::from),
            io_error,
          });
        }
      };
    } else {
      let mut lines = self.lines.iter().peekable();
      let mut line_number = self.line_number + 1;
      loop {
        if lines.peek().is_none() {
          break;
        }
        let mut evaluated = String::new();
        loop {
          if lines.peek().is_none() {
            break;
          }
          let line = lines.next().unwrap();
          line_number += 1;
          evaluated += &evaluator.evaluate_line(line, &argument_map)?;
          if line.last().map(Fragment::continuation).unwrap_or(false) {
            evaluated.pop();
          } else {
            break;
          }
        }
        let mut command = evaluated.as_str();
        let quiet_command = command.starts_with('@');
        if quiet_command {
          command = &command[1..];
        }

        if command == "" {
          continue;
        }

        if configuration.dry_run
          || configuration.verbosity.loquacious()
          || !((quiet_command ^ self.quiet) || configuration.quiet)
        {
          let color = if configuration.highlight {
            configuration.color.command()
          } else {
            configuration.color
          };
          eprintln!("{}", color.stderr().paint(command));
        }

        if configuration.dry_run {
          continue;
        }

        let mut cmd = Command::new(configuration.shell);

        cmd.arg("-cu").arg(command);

        if configuration.quiet {
          cmd.stderr(Stdio::null());
          cmd.stdout(Stdio::null());
        }

        cmd.export_environment_variables(&context.scope, dotenv, exports)?;

        match InterruptHandler::guard(|| cmd.status()) {
          Ok(exit_status) => {
            if let Some(code) = exit_status.code() {
              if code != 0 {
                return Err(RuntimeError::Code {
                  recipe: self.name,
                  line_number: Some(line_number),
                  code,
                });
              }
            } else {
              return Err(error_from_signal(self.name, Some(line_number), exit_status));
            }
          }
          Err(io_error) => {
            return Err(RuntimeError::IoError {
              recipe: self.name,
              io_error,
            });
          }
        };
      }
    }
    Ok(())
  }
}

impl<'a> Display for Recipe<'a> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    if let Some(doc) = self.doc {
      writeln!(f, "# {}", doc)?;
    }
    write!(f, "{}", self.name)?;
    for parameter in &self.parameters {
      write!(f, " {}", parameter)?;
    }
    write!(f, ":")?;
    for dependency in &self.dependencies {
      write!(f, " {}", dependency)?;
    }

    for (i, pieces) in self.lines.iter().enumerate() {
      if i == 0 {
        writeln!(f)?;
      }
      for (j, piece) in pieces.iter().enumerate() {
        if j == 0 {
          write!(f, "    ")?;
        }
        match *piece {
          Fragment::Text { ref text } => write!(f, "{}", text.lexeme)?,
          Fragment::Expression { ref expression, .. } => write!(f, "{{{{{}}}}}", expression)?,
        }
      }
      if i + 1 < self.lines.len() {
        writeln!(f)?;
      }
    }
    Ok(())
  }
}
