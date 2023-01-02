use super::*;
use std::collections::HashSet;

// This regex defines the syntax for a Justfile include statement as `!include <file-path>`
// occurring at the start of a line, and as the only contents of that line
const INCLUDE_REGEX: &str = "^!include\\s+(.+)$";

pub(crate) struct Loader {
  arena: Arena<String>,
}

impl Loader {
  pub(crate) fn new() -> Self {
    Loader {
      arena: Arena::new(),
    }
  }

  pub(crate) fn load<'src>(&'src self, path: &Path) -> RunResult<&'src str> {
    let src = self.perform_load(path, HashSet::new())?;
    Ok(self.arena.alloc(src))
  }

  fn perform_load(&self, path: &Path, seen_paths: HashSet<PathBuf>) -> RunResult<String> {
    println!(
      "Perform load on {}, seen_paths {:?}",
      path.display(),
      seen_paths
    );
    let src = fs::read_to_string(path).map_err(|io_error| Error::Load {
      path: path.to_owned(),
      io_error,
    })?;

    self.process_includes(src, path, seen_paths)
  }

  /// Given the original contents of a Justfile (with include directives), load all the included
  /// paths to produce one String with the contents of all the files concatenate!d. Note that
  /// this does not do any parsing yet (i.e. nothing stops a justfile from including a file
  /// that is not a valid justfile), and that (currently) line numbers in error messages
  /// will reference lines in this concatenated String rather than probably-more-useful
  /// information about the original file an error came from.
  fn process_includes(
    &self,
    original: String,
    current_justfile_path: &Path,
    seen_paths: HashSet<PathBuf>,
  ) -> RunResult<String> {
    let has_final_newline = original.ends_with('\n');

    let include_regexp = Regex::new(INCLUDE_REGEX).unwrap();

    //NOTE this string-processing code can be made a bit cleaner once the Rust std lib Intersperse
    //API is stabilized (https://doc.rust-lang.org/std/iter/struct.Intersperse.html)

    let mut buf = String::new();
    let mut lines = original.lines().peekable();

    while let Some(line) = lines.next() {
      match include_regexp.captures(line) {
        Some(captures) => {
          let path_match = captures.get(1).unwrap();
          let include_path = Path::new(path_match.as_str());
          let included_contents =
            self.process_single_include(current_justfile_path, include_path, &seen_paths)?;
          buf.push_str(&included_contents);
        }
        None => {
          buf.push_str(line);
        }
      };
      if lines.peek().is_some() {
        buf.push('\n');
      }
    }
    if has_final_newline {
      buf.push('\n');
    }

    Ok(buf)
  }

  fn process_single_include(
    &self,
    cur_path: &Path,
    include_path: &Path,
    seen_paths: &HashSet<PathBuf>,
  ) -> RunResult<String> {
    let canonical_path = if include_path.is_relative() {
      let current_dir = cur_path.parent().ok_or(Error::Internal {
        message: format!(
          "Justfile path `{}` has no parent directory",
          include_path.display()
        ),
      })?;

      current_dir
        .join(include_path)
        .canonicalize()
        .map_err(|io_error| Error::Include {
          path: include_path.to_owned(),
          io_error,
        })?
    } else {
      include_path.to_owned()
    };

    if seen_paths.contains(&canonical_path) {
      return Err(Error::IncludeRecursive {
        cur_path: cur_path.to_owned(),
        recursively_included_path: canonical_path,
      });
    }

    let mut seen_paths = seen_paths.clone();
    seen_paths.insert(cur_path.to_owned());

    self.perform_load(&canonical_path, seen_paths)
  }
}

#[cfg(test)]
mod tests {
  use super::{Error, Loader};
  use temptree::temptree;

  #[test]
  fn include_justfile() {
    let justfile_a = r#"
# A comment at the top of the file
!include ./justfile_b

some_recipe: recipe_b
    echo "some recipe"
"#;

    let justfile_b = r#"!include ./subdir/justfile_c

recipe_b: recipe_c
    echo "recipe b"
"#;

    let justfile_c = r#"recipe_c:
    echo "recipe c"
"#;

    let tmp = temptree! {
        justfile: justfile_a,
        justfile_b: justfile_b,
        subdir: {
            justfile_c: justfile_c
        }
    };

    let full_concatenated_output = r#"
# A comment at the top of the file
recipe_c:
    echo "recipe c"


recipe_b: recipe_c
    echo "recipe b"


some_recipe: recipe_b
    echo "some recipe"
"#;

    let loader = Loader::new();

    let justfile_a_path = tmp.path().join("justfile");
    let loader_output = loader.load(&justfile_a_path).unwrap();

    assert_eq!(loader_output, full_concatenated_output);
  }

  #[test]
  fn recursive_includes_fail() {
    let justfile_a = r#"
# A comment at the top of the file
!include ./subdir/justfile_b

some_recipe: recipe_b
    echo "some recipe"
"#;

    let justfile_b = r#"
!include ../justfile

recipe_b:
    echo "recipe b"
"#;
    let tmp = temptree! {
        justfile: justfile_a,
        subdir: {
            justfile_b: justfile_b
        }
    };

    let loader = Loader::new();

    let justfile_a_path = tmp.path().join("justfile");
    let loader_output = loader.load(&justfile_a_path).unwrap_err();

    assert_matches!(loader_output, Error::IncludeRecursive { cur_path, recursively_included_path }
        if cur_path == tmp.path().join("subdir").join("justfile_b") &&
        recursively_included_path == tmp.path().join("justfile")
    );
  }
}
