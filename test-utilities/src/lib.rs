use std::{collections::HashMap, fs, path::Path, process::Output};

pub fn tempdir() -> tempfile::TempDir {
  tempfile::Builder::new()
    .prefix("just-test-tempdir")
    .tempdir()
    .expect("failed to create temporary directory")
}

pub fn assert_success(output: &Output) {
  if !output.status.success() {
    eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    panic!("{}", output.status);
  }
}

pub fn assert_stdout(output: &Output, stdout: &str) {
  assert_success(output);
  assert_eq!(String::from_utf8_lossy(&output.stdout), stdout);
}

pub fn unindent(text: &str) -> String {
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

pub enum Entry {
  File {
    contents: &'static str,
  },
  Dir {
    entries: HashMap<&'static str, Entry>,
  },
}

impl Entry {
  fn instantiate(self, path: &Path) {
    match self {
      Entry::File { contents } => fs::write(path, contents).expect("Failed to write tempfile"),
      Entry::Dir { entries } => {
        fs::create_dir(path).expect("Failed to create tempdir");

        for (name, entry) in entries {
          entry.instantiate(&path.join(name));
        }
      },
    }
  }

  pub fn instantiate_base(base: &Path, entries: HashMap<&'static str, Entry>) {
    for (name, entry) in entries {
      entry.instantiate(&base.join(name));
    }
  }
}

#[macro_export]
macro_rules! entry {
  {
    {
      $($contents:tt)*
    }
  } => {
    $crate::Entry::Dir{entries: $crate::entries!($($contents)*)}
  };
  {
    $contents:expr
  } => {
    $crate::Entry::File{contents: $contents}
  };
}

#[macro_export]
macro_rules! entries {
  {
  } => {
    std::collections::HashMap::new()
  };
  {
    $($name:tt : $contents:tt,)*
  } => {
    {
      use std::collections::HashMap;
      let mut entries: HashMap<&'static str, $crate::Entry> = HashMap::new();

      $(
        entries.insert($crate::name!($name), $crate::entry!($contents));
      )*

      entries
    }
  }
}

#[macro_export]
macro_rules! name {
  {
    $name:ident
  } => {
    stringify!($name)
  };
  {
    $name:literal
  } => {
    $name
  };
}

#[macro_export]
macro_rules! tmptree {
  {
    $($contents:tt)*
  } => {
    {
      let tempdir = $crate::tempdir();

      let entries = $crate::entries!($($contents)*);

      $crate::Entry::instantiate_base(&tempdir.path(), entries);

      tempdir
    }
  }
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

  #[test]
  fn tmptree_file() {
    let tmpdir = tmptree! {
      foo: "bar",
    };

    let contents = fs::read_to_string(tmpdir.path().join("foo")).unwrap();

    assert_eq!(contents, "bar");
  }

  #[test]
  fn tmptree_dir() {
    let tmpdir = tmptree! {
      foo: {
        bar: "baz",
      },
    };

    let contents = fs::read_to_string(tmpdir.path().join("foo/bar")).unwrap();

    assert_eq!(contents, "baz");
  }
}
