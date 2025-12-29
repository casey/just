use super::*;

pub(crate) struct Usage<'a, D> {
  pub(crate) long: bool,
  pub(crate) path: &'a ModulePath,
  pub(crate) recipe: &'a Recipe<'a, D>,
}

impl<D> ColorDisplay for Usage<'_, D> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    write!(
      f,
      "{}{}{} {}",
      color
        .heading()
        .paint(if self.long { "Usage:" } else { "usage:" }),
      if self.long { " " } else { "\n    " },
      color.argument().paint("just"),
      color.argument().paint(&self.path.to_string()),
    )?;

    let options = self.recipe.parameters.iter().any(Parameter::is_option);

    let arguments = self.recipe.parameters.iter().any(|p| !p.is_option());

    if options {
      write!(f, " {}", color.argument().paint("[OPTIONS]"))?;
    }

    for parameter in &self.recipe.parameters {
      if parameter.is_option() {
        continue;
      }

      write!(f, " ")?;

      write!(
        f,
        "{}",
        UsageParameter {
          parameter,
          long: false,
        }
        .color_display(color),
      )?;
    }

    if !self.long {
      return Ok(());
    }

    if arguments {
      writeln!(f)?;
      writeln!(f)?;
      writeln!(f, "{}", color.heading().paint("Arguments:"))?;

      for (i, parameter) in self
        .recipe
        .parameters
        .iter()
        .filter(|p| !p.is_option())
        .enumerate()
      {
        if i > 0 {
          writeln!(f)?;
        }

        write!(f, "  ")?;

        write!(
          f,
          "{}",
          UsageParameter {
            parameter,
            long: true,
          }
          .color_display(color),
        )?;
      }
    }

    if options {
      if arguments {
        writeln!(f)?;
      }

      writeln!(f)?;
      writeln!(f, "{}", color.heading().paint("Options:"))?;
      for (i, parameter) in self
        .recipe
        .parameters
        .iter()
        .filter(|p| p.is_option())
        .enumerate()
      {
        if i > 0 {
          writeln!(f)?;
        }

        write!(f, "  ")?;

        write!(
          f,
          "{}",
          UsageParameter {
            parameter,
            long: true,
          }
          .color_display(color),
        )?;
      }
    }

    Ok(())
  }
}

struct UsageParameter<'a> {
  long: bool,
  parameter: &'a Parameter<'a>,
}

impl ColorDisplay for UsageParameter<'_> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    if self.parameter.is_option() {
      if let Some(short) = self.parameter.short {
        write!(f, "{}", color.option().paint(&format!("-{short}")))?;
      } else {
        write!(f, "  ")?;
      }

      if let Some(long) = &self.parameter.long {
        if self.parameter.short.is_some() {
          write!(f, ", ")?;
        } else {
          write!(f, "  ")?;
        }

        write!(f, "{}", color.option().paint(&format!("--{long}")))?;
      }

      if self.parameter.value.is_none() {
        write!(
          f,
          " {}",
          color.argument().paint(self.parameter.name.lexeme()),
        )?;
      }
    } else {
      if !self.parameter.is_required() {
        write!(f, "{}", color.argument().paint("["))?;
      }

      write!(
        f,
        "{}",
        color.argument().paint(self.parameter.name.lexeme()),
      )?;

      if self.parameter.kind.is_variadic() {
        write!(f, "{}", color.argument().paint("..."))?;
      }

      if !self.parameter.is_required() {
        write!(f, "{}", color.argument().paint("]"))?;
      }
    }

    if !self.long {
      return Ok(());
    }

    if let Some(help) = &self.parameter.help {
      write!(f, " {help}")?;
    }

    if let Some(default) = &self.parameter.default
      && self.parameter.value.is_none()
    {
      write!(f, " [default: {default}]")?;
    }

    if let Some(pattern) = &self.parameter.pattern {
      write!(f, " [pattern: '{pattern}']")?;
    }

    Ok(())
  }
}
