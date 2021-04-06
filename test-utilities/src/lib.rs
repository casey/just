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
