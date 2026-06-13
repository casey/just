use super::*;

pub(crate) struct Element<'a>(pub(crate) &'a str);

impl ColorDisplay for Element<'_> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    let string = color.string();
    let string_escape = color.string_escape();

    write!(f, "{}\"", string.prefix())?;

    let mut escaped = false;

    for c in self.0.chars() {
      let sequence = match c {
        '\\' => Some("\\\\"),
        '"' => Some("\\\""),
        '\n' => Some("\\n"),
        '\r' => Some("\\r"),
        '\t' => Some("\\t"),
        _ => None,
      };

      if let Some(sequence) = sequence {
        if !escaped {
          write!(f, "{}", string_escape.prefix())?;
          escaped = true;
        }
        write!(f, "{sequence}")?;
      } else {
        if escaped {
          write!(f, "{}", string.prefix())?;
          escaped = false;
        }
        write!(f, "{c}")?;
      }
    }

    if escaped {
      write!(f, "{}", string.prefix())?;
    }

    write!(f, "\"{}", string.suffix())
  }
}
