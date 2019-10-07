pub(crate) fn tempdir() -> tempfile::TempDir {
  tempfile::Builder::new()
    .prefix("just-test-tempdir")
    .tempdir()
    .expect("failed to create temporary directory")
}

#[allow(dead_code)]
pub(crate) fn unindent(text: &str) -> String {
  // find line start and end indices
  let mut lines = Vec::new();
  let mut start = 0;
  for (i, c) in text.char_indices() {
    if c == '\n' {
      let end = i + 1;
      lines.push((start, end));
      start = end;
    }
  }

  // if the text isn't newline-terminated, add the final line
  if text.chars().last() != Some('\n') {
    lines.push((start, text.len()));
  }

  // find the longest common indentation
  let mut common_indentation = None;
  for (start, end) in lines.iter().cloned() {
    let line = &text[start..end];

    // skip blank lines
    if blank(line) {
      continue;
    }

    // calculate new common indentation
    common_indentation = match common_indentation {
      Some(common_indentation) => Some(common(common_indentation, indentation(line))),
      None => Some(indentation(line)),
    };
  }

  // if common indentation is present, process the text
  if let Some(common_indentation) = common_indentation {
    if common_indentation != "" {
      let mut output = String::new();

      for (i, (start, end)) in lines.iter().cloned().enumerate() {
        let line = &text[start..end];

        if blank(line) {
          // skip intial and final blank line
          if i != 0 && i != lines.len() - 1 {
            output.push('\n');
          }
        } else {
          // otherwise push the line without the common indentation
          output.push_str(&line[common_indentation.len()..]);
        }
      }

      return output;
    }
  }

  // otherwise just return the input string
  text.to_owned()
}

fn indentation(line: &str) -> &str {
  for (i, c) in line.char_indices() {
    if c != ' ' && c != '\t' {
      return &line[0..i];
    }
  }

  line
}

fn blank(line: &str) -> bool {
  for (i, c) in line.char_indices() {
    if c == ' ' || c == '\t' {
      continue;
    }

    if c == '\n' && i == line.len() - 1 {
      continue;
    }

    return false;
  }

  true
}

fn common<'s>(a: &'s str, b: &'s str) -> &'s str {
  for ((i, ac), bc) in a.char_indices().zip(b.chars()) {
    if ac != bc {
      return &a[0..i];
    }
  }

  a
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn unindents() {
    assert_eq!(unindent("foo"), "foo");
    assert_eq!(unindent("foo\nbar\nbaz\n"), "foo\nbar\nbaz\n");
    assert_eq!(unindent(""), "");
    assert_eq!(unindent("  foo\n  bar"), "foo\nbar");
    assert_eq!(unindent("  foo\n  bar\n\n"), "foo\nbar\n");
  }

  #[test]
  fn indentations() {
    assert_eq!(indentation(""), "");
    assert_eq!(indentation("foo"), "");
    assert_eq!(indentation("   foo"), "   ");
    assert_eq!(indentation("\t\tfoo"), "\t\t");
    assert_eq!(indentation("\t \t foo"), "\t \t ");
  }

  #[test]
  fn blanks() {
    assert!(blank("       \n"));
    assert!(!blank("       foo\n"));
    assert!(blank("\t\t\n"));
  }

  #[test]
  fn commons() {
    assert_eq!(common("foo", "foobar"), "foo");
    assert_eq!(common("foo", "bar"), "");
    assert_eq!(common("", ""), "");
    assert_eq!(common("", "bar"), "");
  }
}
