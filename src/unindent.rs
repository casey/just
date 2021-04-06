pub fn unindent(text: &str) -> String {
  // find line start and end indices
  let mut lines = Vec::new();
  let mut start = 0;
  for (i, c) in text.char_indices() {
    if c == '\n' || i == text.len() - c.len_utf8() {
      let end = i + 1;
      lines.push(&text[start..end]);
      start = end;
    }
  }

  let common_indentation = lines
    .iter()
    .filter(|line| !blank(line))
    .cloned()
    .map(indentation)
    .fold(
      None,
      |common_indentation, line_indentation| match common_indentation {
        Some(common_indentation) => Some(common(common_indentation, line_indentation)),
        None => Some(line_indentation),
      },
    )
    .unwrap_or("");

  let mut replacements = Vec::with_capacity(lines.len());

  for (i, line) in lines.iter().enumerate() {
    let blank = blank(line);
    let first = i == 0;
    let last = i == lines.len() - 1;

    let replacement = match (blank, first, last) {
      (true, false, false) => "\n",
      (true, _, _) => "",
      (false, _, _) => &line[common_indentation.len()..],
    };

    replacements.push(replacement);
  }

  replacements.into_iter().collect()
}

fn indentation(line: &str) -> &str {
  let i = line
    .char_indices()
    .take_while(|(_, c)| matches!(c, ' ' | '\t'))
    .map(|(i, _)| i + 1)
    .last()
    .unwrap_or(0);

  &line[..i]
}

fn blank(line: &str) -> bool {
  line.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n'))
}

fn common<'s>(a: &'s str, b: &'s str) -> &'s str {
  let i = a
    .char_indices()
    .zip(b.chars())
    .take_while(|((_, ac), bc)| ac == bc)
    .map(|((i, c), _)| i + c.len_utf8())
    .last()
    .unwrap_or(0);

  &a[0..i]
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

    assert_eq!(
      unindent(
        "
          hello
          bar
        "
      ),
      "hello\nbar\n"
    );

    assert_eq!(unindent("hello\n  bar\n  foo"), "hello\n  bar\n  foo");

    assert_eq!(
      unindent(
        "

          hello
          bar

        "
      ),
      "\nhello\nbar\n\n"
    );
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
