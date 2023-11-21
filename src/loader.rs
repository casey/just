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

  pub(crate) fn load_and_alloc<'src>(&'src self, path: &Path) -> RunResult<&'src mut String> {
    let src = Self::load_file(path)?;
    Ok(self.arena.alloc(src))
  }

  fn load_file<'a>(path: &Path) -> RunResult<'a, String> {
    fs::read_to_string(path).map_err(|io_error| Error::Load {
      path: path.to_owned(),
      io_error,
    })
  }

  pub(crate) fn load<'src>(&'src self, path: &Path) -> RunResult<&'src str> {
    Ok(self.load_and_alloc(path)?)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, temptree::temptree};

  #[test]
  fn include_justfile() {
    let justfile_a = r#"
# A comment at the top of the file
!include ./justfile_b

#some_recipe: recipe_b
some_recipe:
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

    let loader = Loader::new();

    let justfile_a_path = tmp.path().join("justfile");
    let compilation = Compiler::compile(true, &loader, &justfile_a_path).unwrap();

    assert_eq!(compilation.src(), justfile_a);
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
    let loader_output = Compiler::compile(true, &loader, &justfile_a_path).unwrap_err();

    assert_matches!(loader_output, Error::CircularInclude { current, include }
        if current == tmp.path().join("subdir").join("justfile_b").lexiclean() &&
        include == tmp.path().join("justfile").lexiclean()
    );
  }
}
