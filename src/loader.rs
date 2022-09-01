use super::*;

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
    let src = fs::read_to_string(path).map_err(|io_error| Error::Load {
      path: path.to_owned(),
      io_error,
    })?;

    // Parse include directives.
    // Include directives are lines that look like:
    // #include "relative/path/to/file"
    let include_regexp = Regex::new(r#"^#include "([^"]+)"$"#).unwrap();

    let parent = path.parent().map_or_else(|| Path::new(""), |p| p);

    let mut buf = String::new();
    let lines = src.lines();
    for line in lines {
      if let Some(captures) = include_regexp.captures(line) {
        // safe because we don't get here without a match
        let filename = captures.get(1).unwrap().as_str();
        let mut include_path = PathBuf::from(&parent);
        include_path.push(filename);

        let contents = fs::read_to_string(&include_path).map_err(|io_error| Error::Load {
          path: include_path.clone(),
          io_error,
        })?;
        buf.push_str(&contents);
      } else {
        buf.push_str(line);
      }
      buf.push('\n');
    }

    Ok(self.arena.alloc(buf))
  }
}
