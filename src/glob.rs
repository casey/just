use super::*;
use std::ffi::OsStr;

/// Check if a path contains glob pattern characters.
///
/// This function works at the byte/UTF-16 level to support non-Unicode paths.
/// It checks for the presence of `*` wildcard characters in any path component.
///
/// # Why Custom Implementation?
///
/// Existing Rust glob crates (glob, globset, wax) require UTF-8 conversion
/// via `to_str()` or `to_string_lossy()`, which breaks Just's commitment to
/// supporting non-Unicode filesystem paths on all platforms.
///
/// This implementation works directly with `OsStr` at the platform-native level:
/// - Unix: Operates on byte sequences via `OsStrExt::as_bytes()`
/// - Windows: Operates on UTF-16 sequences via `OsStrExt::encode_wide()`
pub(crate) fn has_glob_pattern(path: &Path) -> bool {
  path
    .components()
    .any(|component| contains_asterisk(component.as_os_str()))
}

/// Expand a glob pattern to a sorted list of matching file paths.
///
/// # Supported Patterns
///
/// Currently supports only `*` wildcard:
/// - `*` matches zero or more characters except path separators
///
/// # Behavior
///
/// - Returns matches sorted lexicographically for deterministic ordering
/// - Only returns regular files (not directories or symlinks to directories)
/// - Follows symbolic links (standard `fs::read_dir()` behavior)
/// - Returns empty vector if no matches found
///
/// # Examples
///
/// ```
/// // Pattern: '.just/*.justfile'
/// // Matches: ['.just/bar.justfile', '.just/foo.justfile']
/// ```
pub(crate) fn expand_glob_pattern(pattern: &Path) -> RunResult<'static, Vec<PathBuf>> {
  // Split pattern into directory and filename pattern
  let (directory, filename_pattern) = if let Some(parent) = pattern.parent() {
    if let Some(filename) = pattern.file_name() {
      (parent, filename)
    } else {
      // Pattern ends with a directory separator, no filename pattern
      return Ok(Vec::new());
    }
  } else {
    // No directory component, pattern is just a filename
    (Path::new("."), pattern.as_os_str())
  };

  // Check if filename pattern actually contains wildcards
  if !contains_asterisk(filename_pattern) {
    // No wildcards in filename, should not have been called
    return Ok(Vec::new());
  }

  // Read directory entries
  let entries = match fs::read_dir(directory) {
    Ok(entries) => entries,
    Err(io_error) => {
      if io_error.kind() == io::ErrorKind::NotFound {
        // Directory doesn't exist, return empty matches
        return Ok(Vec::new());
      } else {
        return Err(Error::GlobReadDirectory {
          path: directory.into(),
          io_error,
        });
      }
    }
  };

  let mut matches = Vec::new();

  for entry in entries {
    let entry = entry.map_err(|io_error| Error::GlobReadDirectory {
      path: directory.into(),
      io_error,
    })?;

    let filename = entry.file_name();

    // Match filename against pattern
    if match_pattern(&filename, filename_pattern) {
      let full_path = directory.join(filename);

      // Only include regular files
      match full_path.metadata() {
        Ok(metadata) if metadata.is_file() => {
          matches.push(full_path);
        }
        Ok(_) => {
          // Not a regular file (directory, symlink to dir, etc), skip
        }
        Err(io_error) => {
          if io_error.kind() != io::ErrorKind::NotFound {
            // File disappeared or permission denied - fail fast
            return Err(Error::GlobReadDirectory {
              path: full_path,
              io_error,
            });
          }
          // File disappeared between readdir and metadata, skip
        }
      }
    }
  }

  // Sort for deterministic ordering
  matches.sort();

  Ok(matches)
}

/// Match a filename against a pattern containing `*` wildcards.
///
/// Platform-specific implementation that works at the byte/UTF-16 level
/// without requiring UTF-8 conversion.
///
/// # Pattern Syntax
///
/// - `*` matches zero or more characters (including none)
/// - All other characters match literally
/// - Case-sensitive matching
///
/// # Examples
///
/// - `match_pattern("foo.just", "*.just")` → `true`
/// - `match_pattern("bar.justfile", "*.just")` → `false`
/// - `match_pattern("test.just", "test.*")` → `true`
fn match_pattern(filename: &OsStr, pattern: &OsStr) -> bool {
  #[cfg(unix)]
  {
    use std::os::unix::ffi::OsStrExt;
    match_with_wildcards(filename.as_bytes(), pattern.as_bytes(), b'*')
  }

  #[cfg(windows)]
  {
    use std::os::windows::ffi::OsStrExt;
    let filename_wide: Vec<u16> = filename.encode_wide().collect();
    let pattern_wide: Vec<u16> = pattern.encode_wide().collect();
    match_with_wildcards(&filename_wide, &pattern_wide, b'*' as u16)
  }

  #[cfg(not(any(unix, windows)))]
  {
    // Fallback for other platforms: attempt UTF-8 matching
    // This is not ideal but covers edge cases
    if let (Some(f), Some(p)) = (filename.to_str(), pattern.to_str()) {
      match_with_wildcards(f.as_bytes(), p.as_bytes(), b'*')
    } else {
      false
    }
  }
}

/// Generic wildcard matching algorithm.
///
/// Matches text against a pattern containing wildcards, operating on
/// any sequence of comparable elements (bytes, UTF-16 code units, etc.).
///
/// # Algorithm
///
/// Uses a simple iterative algorithm:
/// - `*` matches zero or more characters
/// - Tries to match greedily from left to right
/// - Backtracks when needed to find a valid match
///
/// # Complexity
///
/// O(n * m) where n is text length and m is pattern length.
/// Worst case: pattern like `*a*b*c` with text that has many a's, b's, c's.
fn match_with_wildcards<T: Eq>(text: &[T], pattern: &[T], wildcard: T) -> bool {
  let mut text_idx = 0;
  let mut pattern_idx = 0;
  let mut star_idx = None;
  let mut match_idx = 0;

  while text_idx < text.len() {
    if pattern_idx < pattern.len() && pattern[pattern_idx] == wildcard {
      // Found a wildcard, record position and try matching rest
      star_idx = Some(pattern_idx);
      match_idx = text_idx;
      pattern_idx += 1;
    } else if pattern_idx < pattern.len() && pattern[pattern_idx] == text[text_idx] {
      // Characters match, advance both
      text_idx += 1;
      pattern_idx += 1;
    } else if let Some(star) = star_idx {
      // Mismatch, backtrack to last wildcard
      pattern_idx = star + 1;
      match_idx += 1;
      text_idx = match_idx;
    } else {
      // No match and no wildcard to backtrack to
      return false;
    }
  }

  // Consume any trailing wildcards in pattern
  while pattern_idx < pattern.len() && pattern[pattern_idx] == wildcard {
    pattern_idx += 1;
  }

  // Match succeeds if we've consumed both text and pattern
  pattern_idx == pattern.len()
}

/// Check if an OsStr contains an asterisk character.
///
/// Platform-specific implementation to avoid UTF-8 conversion.
#[cfg(unix)]
fn contains_asterisk(os_str: &OsStr) -> bool {
  use std::os::unix::ffi::OsStrExt;
  os_str.as_bytes().contains(&b'*')
}

#[cfg(windows)]
fn contains_asterisk(os_str: &OsStr) -> bool {
  use std::os::windows::ffi::OsStrExt;
  os_str.encode_wide().any(|c| c == b'*' as u16)
}

#[cfg(not(any(unix, windows)))]
fn contains_asterisk(os_str: &OsStr) -> bool {
  // Fallback: try UTF-8 conversion
  os_str.to_str().map_or(false, |s| s.contains('*'))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_has_glob_pattern() {
    assert!(has_glob_pattern(Path::new("*.just")));
    assert!(has_glob_pattern(Path::new(".just/*.justfile")));
    assert!(has_glob_pattern(Path::new("foo-*.just")));
    assert!(!has_glob_pattern(Path::new("foo.just")));
    assert!(!has_glob_pattern(Path::new(".just/foo.just")));
  }

  #[test]
  fn test_match_with_wildcards_bytes() {
    // Basic wildcard matching
    assert!(match_with_wildcards(b"foo.just", b"*.just", b'*'));
    assert!(match_with_wildcards(b"bar.just", b"*.just", b'*'));
    assert!(!match_with_wildcards(b"foo.justfile", b"*.just", b'*'));

    // Wildcard at start
    assert!(match_with_wildcards(b"test.just", b"*st.just", b'*'));
    assert!(match_with_wildcards(b"test.just", b"*est.just", b'*'));

    // Wildcard in middle
    assert!(match_with_wildcards(b"foo-bar.just", b"foo-*.just", b'*'));
    assert!(match_with_wildcards(b"foo-.just", b"foo-*.just", b'*'));

    // Multiple wildcards
    assert!(match_with_wildcards(b"a-b-c.just", b"*-*-*.just", b'*'));
    assert!(match_with_wildcards(b"abc.just", b"*bc.just", b'*'));

    // Wildcard matches empty
    assert!(match_with_wildcards(b"foo", b"foo*", b'*'));
    assert!(match_with_wildcards(b"foo", b"*foo", b'*'));

    // No wildcards (literal match)
    assert!(match_with_wildcards(b"exact", b"exact", b'*'));
    assert!(!match_with_wildcards(b"exact", b"other", b'*'));

    // Edge cases
    assert!(match_with_wildcards(b"", b"*", b'*'));
    assert!(match_with_wildcards(b"anything", b"*", b'*'));
    assert!(!match_with_wildcards(b"foo", b"", b'*'));
  }

  #[test]
  fn test_match_pattern() {
    use std::ffi::OsStr;

    // Basic matching
    assert!(match_pattern(OsStr::new("foo.just"), OsStr::new("*.just")));
    assert!(match_pattern(
      OsStr::new("bar.justfile"),
      OsStr::new("*.justfile")
    ));
    assert!(!match_pattern(
      OsStr::new("foo.just"),
      OsStr::new("*.justfile")
    ));

    // Prefix matching
    assert!(match_pattern(
      OsStr::new("test-foo.just"),
      OsStr::new("test-*.just")
    ));

    // No wildcards
    assert!(match_pattern(OsStr::new("exact.just"), OsStr::new("exact.just")));
    assert!(!match_pattern(
      OsStr::new("exact.just"),
      OsStr::new("other.just")
    ));
  }

  #[test]
  fn test_contains_asterisk() {
    use std::ffi::OsStr;

    assert!(contains_asterisk(OsStr::new("*.just")));
    assert!(contains_asterisk(OsStr::new("foo-*.just")));
    assert!(contains_asterisk(OsStr::new("*")));
    assert!(!contains_asterisk(OsStr::new("foo.just")));
    assert!(!contains_asterisk(OsStr::new("")));
  }

  #[test]
  #[cfg(unix)]
  fn test_non_utf8_filenames() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    // Create OsStr with invalid UTF-8
    let invalid_utf8 = OsStr::from_bytes(&[0xff, 0xfe, b'.', b'j', b'u', b's', b't']);
    let pattern = OsStr::new("*.just");

    // Should match even though filename isn't valid UTF-8
    assert!(match_pattern(invalid_utf8, pattern));

    // Check that has_glob_pattern works with invalid UTF-8 patterns
    let invalid_pattern = Path::new(OsStr::from_bytes(&[b'*', 0xff, 0xfe]));
    assert!(has_glob_pattern(invalid_pattern));
  }
}
