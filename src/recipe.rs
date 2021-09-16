use crate::common::*;

use std::process::{ExitStatus, Stdio};

/// Return a `Error::Signal` if the process was terminated by a signal,
/// otherwise return an `Error::UnknownFailure`
fn error_from_signal(recipe: &str, line_number: Option<usize>, exit_status: ExitStatus) -> Error {
  match Platform::signal_from_exit_status(exit_status) {
    Some(signal) => Error::Signal {
      recipe,
      line_number,
      signal,
    },
    None => Error::Unknown {
      recipe,
      line_number,
    },
  }
}

/// A recipe, e.g. `foo: bar baz`
#[derive(PartialEq, Debug, Clone)]
pub(crate) struct Recipe<'src, D = Dependency<'src>> {
  pub(crate) body: Vec<Line<'src>>,
  pub(crate) dependencies: Vec<D>,
  pub(crate) doc: Option<&'src str>,
  pub(crate) name: Name<'src>,
  pub(crate) parameters: Vec<Parameter<'src>>,
  pub(crate) private: bool,
  pub(crate) quiet: bool,
  pub(crate) shebang: bool,
  pub(crate) priors: usize,
}

impl<'src, D> Recipe<'src, D> {
  pub(crate) fn argument_range(&self) -> RangeInclusive<usize> {
    self.min_arguments()..=self.max_arguments()
  }

  pub(crate) fn min_arguments(&self) -> usize {
    self
      .parameters
      .iter()
      .filter(|p| p.default.is_none() && p.kind != ParameterKind::Star)
      .count()
  }

  pub(crate) fn max_arguments(&self) -> usize {
    if self.parameters.iter().any(|p| p.kind.is_variadic()) {
      usize::max_value() - 1
    } else {
      self.parameters.len()
    }
  }

  pub(crate) fn name(&self) -> &'src str {
    self.name.lexeme()
  }

  pub(crate) fn line_number(&self) -> usize {
    self.name.line
  }

  pub(crate) fn public(&self) -> bool {
    !self.private
  }

  pub(crate) fn run<'run>(
    &self,
    context: &RecipeContext<'src, 'run>,
    dotenv: &BTreeMap<String, String>,
    scope: Scope<'src, 'run>,
    search: &'run Search,
    positional: &[String],
  ) -> RunResult<'src, ()> {
    let config = &context.config;

    if config.verbosity.loquacious() {
      let color = config.color.stderr().banner();
      eprintln!(
        "{}===> Running recipe `{}`...{}",
        color.prefix(),
        self.name,
        color.suffix()
      );
    }

    let mut evaluator =
      Evaluator::recipe_evaluator(context.config, dotenv, &scope, context.settings, search);

    if self.shebang {
      let mut evaluated_lines = vec![];
      for line in &self.body {
        evaluated_lines.push(evaluator.evaluate_line(line, false)?);
      }

      if config.verbosity.loud() && (config.dry_run || self.quiet) {
        for line in &evaluated_lines {
          eprintln!("{}", line);
        }
      }

      if config.dry_run {
        return Ok(());
      }

      let shebang_line = evaluated_lines.first().ok_or_else(|| Error::Internal {
        message: "evaluated_lines was empty".to_owned(),
      })?;

      let shebang = Shebang::new(shebang_line).ok_or_else(|| Error::Internal {
        message: format!("bad shebang line: {}", shebang_line),
      })?;

      let tmp = tempfile::Builder::new()
        .prefix("just")
        .tempdir()
        .map_err(|error| Error::TmpdirIo {
          recipe: self.name(),
          io_error: error,
        })?;
      let mut path = tmp.path().to_path_buf();

      path.push(shebang.script_filename(self.name()));

      {
        let mut f = fs::File::create(&path).map_err(|error| Error::TmpdirIo {
          recipe: self.name(),
          io_error: error,
        })?;
        let mut text = String::new();

        if shebang.include_shebang_line() {
          text += &evaluated_lines[0];
        } else {
          text += "\n";
        }

        text += "\n";
        // add blank lines so that lines in the generated script have the same line
        // number as the corresponding lines in the justfile
        for _ in 1..(self.line_number() + 2) {
          text += "\n";
        }
        for line in &evaluated_lines[1..] {
          text += line;
          text += "\n";
        }

        if config.verbosity.grandiloquent() {
          eprintln!("{}", config.color.doc().stderr().paint(&text));
        }

        f.write_all(text.as_bytes())
          .map_err(|error| Error::TmpdirIo {
            recipe: self.name(),
            io_error: error,
          })?;
      }

      // make the script executable
      Platform::set_execute_permission(&path).map_err(|error| Error::TmpdirIo {
        recipe: self.name(),
        io_error: error,
      })?;

      // create a command to run the script
      let mut command =
        Platform::make_shebang_command(&path, &context.search.working_directory, shebang).map_err(
          |output_error| Error::Cygpath {
            recipe: self.name(),
            output_error,
          },
        )?;

      if context.settings.positional_arguments {
        command.args(positional);
      }

      command.export(context.settings, dotenv, &scope);

      // run it!
      match InterruptHandler::guard(|| command.status()) {
        Ok(exit_status) => {
          if let Some(code) = exit_status.code() {
            if code != 0 {
              return Err(Error::Code {
                recipe: self.name(),
                line_number: None,
                code,
              });
            }
          } else {
            return Err(error_from_signal(self.name(), None, exit_status));
          }
        }
        Err(io_error) => {
          return Err(Error::Shebang {
            recipe: self.name(),
            command: shebang.interpreter.to_owned(),
            argument: shebang.argument.map(String::from),
            io_error,
          });
        }
      };
    } else {
      let mut lines = self.body.iter().peekable();
      let mut line_number = self.line_number() + 1;
      loop {
        if lines.peek().is_none() {
          break;
        }
        let mut evaluated = String::new();
        let mut continued = false;
        let quiet_command = lines.peek().map(|line| line.is_quiet()).unwrap_or(false);
        let infallable_command = lines
          .peek()
          .map(|line| line.is_infallable())
          .unwrap_or(false);
        loop {
          if lines.peek().is_none() {
            break;
          }
          let line = lines.next().unwrap();
          line_number += 1;
          evaluated += &evaluator.evaluate_line(line, continued)?;
          if line.is_continuation() {
            continued = true;
            evaluated.pop();
          } else {
            break;
          }
        }
        let mut command = evaluated.as_str();
        if quiet_command || infallable_command {
          command = &command[1..];
        }

        if command.is_empty() {
          continue;
        }

        if config.dry_run
          || config.verbosity.loquacious()
          || !((quiet_command ^ self.quiet) || config.verbosity.quiet())
        {
          let color = if config.highlight {
            config.color.command()
          } else {
            config.color
          };
          eprintln!("{}", color.stderr().paint(command));
        }

        if config.dry_run {
          continue;
        }

        let mut cmd = context.settings.shell_command(config);

        cmd.current_dir(&context.search.working_directory);

        cmd.arg(command);

        if context.settings.positional_arguments {
          cmd.arg(self.name.lexeme());
          cmd.args(positional);
        }

        if config.verbosity.quiet() {
          cmd.stderr(Stdio::null());
          cmd.stdout(Stdio::null());
        }

        cmd.export(context.settings, dotenv, &scope);

        match InterruptHandler::guard(|| cmd.status()) {
          Ok(exit_status) => {
            if let Some(code) = exit_status.code() {
              if code != 0 && !infallable_command {
                return Err(Error::Code {
                  recipe: self.name(),
                  line_number: Some(line_number),
                  code,
                });
              }
            } else {
              return Err(error_from_signal(
                self.name(),
                Some(line_number),
                exit_status,
              ));
            }
          }
          Err(io_error) => {
            return Err(Error::Io {
              recipe: self.name(),
              io_error,
            });
          }
        };
      }
    }
    Ok(())
  }
}

impl<'src, D> Keyed<'src> for Recipe<'src, D> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}

impl<'src, D: Display> ColorDisplay for Recipe<'src, D> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> Result<(), fmt::Error> {
    if let Some(doc) = self.doc {
      writeln!(f, "# {}", doc)?;
    }

    if self.quiet {
      write!(f, "@{}", self.name)?;
    } else {
      write!(f, "{}", self.name)?;
    }

    for parameter in &self.parameters {
      write!(f, " {}", parameter.color_display(color))?;
    }
    write!(f, ":")?;

    for (i, dependency) in self.dependencies.iter().enumerate() {
      if i == self.priors {
        write!(f, " &&")?;
      }

      write!(f, " {}", dependency)?;
    }

    for (i, line) in self.body.iter().enumerate() {
      if i == 0 {
        writeln!(f)?;
      }
      for (j, fragment) in line.fragments.iter().enumerate() {
        if j == 0 {
          write!(f, "    ")?;
        }
        match fragment {
          Fragment::Text { token } => write!(f, "{}", token.lexeme())?,
          Fragment::Interpolation { expression, .. } => write!(f, "{{{{ {} }}}}", expression)?,
        }
      }
      if i + 1 < self.body.len() {
        writeln!(f)?;
      }
    }
    Ok(())
  }
}
