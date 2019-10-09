use crate::common::*;

pub(crate) fn write_message_context(
  f: &mut Formatter,
  color: Color,
  text: &str,
  offset: usize,
  line: usize,
  column: usize,
  width: usize,
) -> Result<(), fmt::Error> {
  let width = if width == 0 { 1 } else { width };

  let line_number = line.ordinal();
  match text.lines().nth(line) {
    Some(line) => {
      let mut i = 0;
      let mut space_column = 0;
      let mut space_line = String::new();
      let mut space_width = 0;
      for c in line.chars() {
        if c == '\t' {
          space_line.push_str("    ");
          if i < column {
            space_column += 4;
          }
          if i >= column && i < column + width {
            space_width += 4;
          }
        } else {
          if i < column {
            space_column += UnicodeWidthChar::width(c).unwrap_or(0);
          }
          if i >= column && i < column + width {
            space_width += UnicodeWidthChar::width(c).unwrap_or(0);
          }
          space_line.push(c);
        }
        i += c.len_utf8();
      }
      let line_number_width = line_number.to_string().len();
      writeln!(f, "{0:1$} |", "", line_number_width)?;
      writeln!(f, "{} | {}", line_number, space_line)?;
      write!(f, "{0:1$} |", "", line_number_width)?;
      write!(
        f,
        " {0:1$}{2}{3:^<4$}{5}",
        "",
        space_column,
        color.prefix(),
        "",
        space_width,
        color.suffix()
      )?;
    }
    None => {
      if offset != text.len() {
        write!(
          f,
          "internal error: Error has invalid line number: {}",
          line_number
        )?
      }
    }
  }
  Ok(())
}
