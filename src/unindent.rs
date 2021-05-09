use std::str::CharIndices;

trait Unindent<'a>: ToOwned {
  type ItemIndices: Iterator<Item = (usize, Option<char>, bool)>;

  fn unindent(&'a self) -> Self::Owned {
    let lines = self.split_lines();

    let common_indentation = Self::common_indentation(&lines);

    let mut replacements = Vec::with_capacity(lines.len());

    for (i, line) in lines.iter().enumerate() {
      let blank = line.blank();
      let first = i == 0;
      let last = i == lines.len() - 1;

      let replacement = match (blank, first, last) {
        (true, false, false) => Self::from_str("\n"),
        (true, _, _) => Self::from_str(""),
        (false, _, _) => line.slice(common_indentation.len(), line.len()),
      };

      replacements.push(replacement);
    }

    Self::join(replacements)
  }

  fn common_indentation(lines: &[&'a Self]) -> &'a Self {
    lines
      .iter()
      .filter(|line| !line.blank())
      .map(|line| line.indentation())
      .fold(None, |acc, current| match acc {
        None => Some(current),
        Some(acc) => Some(Self::common(acc, current)),
      })
      .unwrap_or(Self::from_str(""))
  }

  fn split_lines(&'a self) -> Vec<&Self> {
    let mut lines = Vec::new();
    let mut start = 0;
    for (i, c, last) in self.item_indices() {
      if c == Some('\n') || last {
        let end = i + 1;
        lines.push(self.slice(start, end));
        start = end;
      }
    }
    lines
  }

  fn item_indices(&'a self) -> Self::ItemIndices;

  fn slice(&self, start: usize, end: usize) -> &Self;

  fn from_str(s: &'static str) -> &'a Self;

  fn len(&self) -> usize;

  fn join(items: Vec<&Self>) -> Self::Owned;

  fn blank(&'a self) -> bool {
    self.item_indices().all(|(_, o, _)| {
      o.map(|c| matches!(c, ' ' | '\t' | '\r' | '\n'))
        .unwrap_or(false)
    })
  }

  fn indentation(&'a self) -> &Self {
    let i = self
      .item_indices()
      .take_while(|(_, o, _)| o.map(|c| matches!(c, ' ' | '\t')).unwrap_or(false))
      .map(|(i, _, _)| i + 1)
      .last()
      .unwrap_or(0);

    self.slice(0, i)
  }

  fn common(a: &'a Self, b: &'a Self) -> &'a Self {
    let i = a
      .item_indices()
      .zip(b.item_indices())
      .take_while(|((_, ac, _), (_, bc, _))| ac.is_some() && ac == bc)
      .map(|((i, c, _), _)| i + c.unwrap().len_utf8())
      .last()
      .unwrap_or(0);

    a.slice(0, i)
  }
}

struct StrItemIndices<'a> {
  inner: CharIndices<'a>,
}

impl<'a> Iterator for StrItemIndices<'a> {
  type Item = (usize, Option<char>, bool);

  fn next(&mut self) -> Option<(usize, Option<char>, bool)> {
    let (i, c) = self.inner.next()?;
    let last = self.inner.as_str().is_empty();
    Some((i, Some(c), last))
  }
}

impl<'a> Unindent<'a> for str {
  type ItemIndices = StrItemIndices<'a>;

  fn slice(&self, start: usize, end: usize) -> &Self {
    &self[start..end]
  }

  fn item_indices(&'a self) -> Self::ItemIndices {
    Self::ItemIndices {
      inner: self.char_indices(),
    }
  }

  fn from_str(s: &'static str) -> &'a Self {
    s
  }

  fn len(&self) -> usize {
    self.len()
  }

  fn join(items: Vec<&Self>) -> Self::Owned {
    items.into_iter().collect()
  }
}

#[must_use]
pub fn unindent(text: &str) -> String {
  let lines = split_lines(text);
  let common_indentation = get_common_indentation(lines.iter().cloned());

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

pub fn split_lines(text: &str) -> Vec<&str> {
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
  lines
}

pub fn get_common_indentation<'src, I>(lines: I) -> &'src str
where
  I: Iterator<Item = &'src str>,
{
  lines
    .filter(|line| !blank(line))
    .map(indentation)
    .fold(None, |acc, current| match acc {
      None => Some(current),
      Some(acc) => Some(common(acc, current)),
    })
    .unwrap_or("")
}

pub fn indentation(line: &str) -> &str {
  let i = line
    .char_indices()
    .take_while(|(_, c)| matches!(c, ' ' | '\t'))
    .map(|(i, _)| i + 1)
    .last()
    .unwrap_or(0);

  &line[..i]
}

pub fn blank(line: &str) -> bool {
  line.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n'))
}

pub fn common<'s>(a: &'s str, b: &'s str) -> &'s str {
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
    assert_eq!("foo".unindent(), "foo");
    assert_eq!("foo\nbar\nbaz\n".unindent(), "foo\nbar\nbaz\n");
    assert_eq!("".unindent(), "");
    assert_eq!("  foo\n  bar".unindent(), "foo\nbar");
    assert_eq!("  foo\n  bar\n\n".unindent(), "foo\nbar\n");

    assert_eq!(
      "
          hello
          bar
        "
      .unindent(),
      "hello\nbar\n"
    );

    assert_eq!("hello\n  bar\n  foo".unindent(), "hello\n  bar\n  foo");

    assert_eq!(
      "

          hello
          bar

        "
      .unindent(),
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
