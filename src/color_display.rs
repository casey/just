use super::*;

pub(crate) trait ColorDisplay {
  fn color_display(&self, color: Color) -> Wrapper
  where
    Self: Sized,
  {
    Wrapper {
      color,
      indentation: "",
      value: self,
    }
  }

  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result;
}

pub(crate) struct Wrapper<'a> {
  color: Color,
  indentation: &'static str,
  value: &'a dyn ColorDisplay,
}

impl Wrapper<'_> {
  /// Indent every non-blank line of the wrapped value's output, omitting any
  /// trailing newline
  pub(crate) fn indented(self, indentation: &'static str) -> Self {
    Self {
      indentation,
      ..self
    }
  }
}

impl Display for Wrapper<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use fmt::Write as _;

    if self.indentation.is_empty() {
      self.value.fmt(f, self.color)
    } else {
      let mut indenter = Indenter {
        formatter: f,
        indentation: self.indentation,
        needs_indentation: true,
        pending_newlines: 0,
      };
      write!(
        indenter,
        "{}",
        Wrapper {
          color: self.color,
          indentation: "",
          value: self.value,
        }
      )
    }
  }
}

/// A `fmt::Write` adapter that indents every non-blank line by `indentation`
/// and drops trailing newlines
struct Indenter<'a, 'b> {
  formatter: &'a mut Formatter<'b>,
  indentation: &'static str,
  needs_indentation: bool,
  pending_newlines: usize,
}

impl fmt::Write for Indenter<'_, '_> {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    for line in s.split_inclusive('\n') {
      let (content, newline) = match line.strip_suffix('\n') {
        Some(content) => (content, true),
        None => (line, false),
      };

      if !content.is_empty() {
        for _ in 0..self.pending_newlines {
          self.formatter.write_char('\n')?;
        }
        self.pending_newlines = 0;

        if self.needs_indentation {
          self.formatter.write_str(self.indentation)?;
          self.needs_indentation = false;
        }

        self.formatter.write_str(content)?;
      }

      if newline {
        self.pending_newlines += 1;
        self.needs_indentation = true;
      }
    }

    Ok(())
  }
}
