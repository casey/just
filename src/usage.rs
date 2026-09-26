use super::*;

#[derive(PartialEq)]
pub(crate) enum Mode {
  Module,
  Recipe,
  Short,
}

pub(crate) struct Usage<'a> {
  pub(crate) mode: Mode,
  pub(crate) path: &'a Modulepath,
  pub(crate) recipe: &'a Recipe<'a>,
}

impl ColorDisplay for Usage<'_> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    let indentation = match self.mode {
      Mode::Module => "    ",
      Mode::Recipe | Mode::Short => "",
    };

    match self.mode {
      Mode::Module => {
        if let Some(doc) = self.recipe.doc() {
          for line in doc.lines() {
            writeln!(
              f,
              "{indentation}{}",
              color.doc().paint(&format!("# {line}"))
            )?;
          }
        }
      }
      Mode::Recipe => {
        write!(f, "{} ", color.heading().paint("Usage:"))?;
      }
      Mode::Short => {
        write!(f, "{}\n    ", color.heading().paint("usage:"))?;
      }
    }

    write!(
      f,
      "{indentation}{} {}",
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

    if self.mode == Mode::Short {
      return Ok(());
    }

    if arguments {
      writeln!(f)?;

      if self.mode == Mode::Recipe {
        writeln!(f)?;
        writeln!(f, "{}", color.heading().paint("Arguments:"))?;
      }

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

        write!(
          f,
          "{indentation}  {}",
          UsageParameter {
            parameter,
            long: true,
          }
          .color_display(color),
        )?;
      }
    }

    if options {
      writeln!(f)?;

      if self.mode == Mode::Recipe {
        writeln!(f)?;
        writeln!(f, "{}", color.heading().paint("Options:"))?;
      }

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

        write!(
          f,
          "{indentation}  {}",
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

      if self.parameter.value.is_none() && !self.parameter.flag {
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
      write!(f, " [pattern: ")?;

      for (i, original) in pattern.originals().enumerate() {
        if i > 0 {
          write!(f, " | ")?;
        }
        write!(f, "'{original}'")?;
      }

      write!(f, "]")?;
    }

    Ok(())
  }
}
