use super::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Token<'src> {
  pub(crate) column: usize,
  pub(crate) kind: TokenKind,
  pub(crate) length: usize,
  pub(crate) line: usize,
  pub(crate) offset: usize,
  pub(crate) path: &'src Path,
  pub(crate) src: &'src str,
}

impl<'src> Token<'src> {
  pub(crate) fn lexeme(&self) -> &'src str {
    &self.src[self.offset..self.offset + self.length]
  }

  pub(crate) fn error(&self, kind: CompileErrorKind<'src>) -> CompileError<'src> {
    CompileError::new(*self, kind)
  }
}

impl<'src> ColorDisplay for Token<'src> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    let width = if self.length == 0 { 1 } else { self.length };

    let line_number = self.line.ordinal();
    match self.src.lines().nth(self.line) {
      Some(line) => {
        let mut i = 0;
        let mut space_column = 0;
        let mut space_line = String::new();
        let mut space_width = 0;
        for c in line.chars() {
          if c == '\t' {
            space_line.push_str("    ");
            if i < self.column {
              space_column += 4;
            }
            if i >= self.column && i < self.column + width {
              space_width += 4;
            }
          } else {
            if i < self.column {
              space_column += UnicodeWidthChar::width(c).unwrap_or(0);
            }
            if i >= self.column && i < self.column + width {
              space_width += UnicodeWidthChar::width(c).unwrap_or(0);
            }
            space_line.push(c);
          }
          i += c.len_utf8();
        }
        let line_number_width = line_number.to_string().len();

        writeln!(
          f,
          "{:width$}{} {}:{}:{}",
          "",
          color.context().paint("-->"),
          self.path.display(),
          line_number,
          self.column.ordinal(),
          width = line_number_width
        )?;
        writeln!(
          f,
          "{:width$} {}",
          "",
          color.context().paint("|"),
          width = line_number_width
        )?;
        writeln!(
          f,
          "{} {space_line}",
          color.context().paint(&format!("{line_number} |"))
        )?;
        write!(
          f,
          "{:width$} {}",
          "",
          color.context().paint("|"),
          width = line_number_width
        )?;
        write!(
          f,
          " {0:1$}{2}{3:^<4$}{5}",
          "",
          space_column,
          color.prefix(),
          "",
          space_width.max(1),
          color.suffix()
        )?;
      }
      None => {
        if self.offset != self.src.len() {
          write!(
            f,
            "internal error: Error has invalid line number: {line_number}"
          )?;
        }
      }
    }

    Ok(())
  }
}
