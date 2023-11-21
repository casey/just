use super::*;
use std::collections::{HashSet, VecDeque};

pub(crate) struct Loader {
  arena: Arena<String>,
  unstable: bool,
}

impl Loader {
  pub(crate) fn new(unstable: bool) -> Self {
    Loader {
      arena: Arena::new(),
      unstable,
    }
  }

  pub(crate) fn load_and_compile(&self, path: &Path) -> RunResult<Compilation> {
    let root_src = self.load_and_alloc(path)?;
    let root_ast = Compiler::parse(root_src)?;
    let root_imports = Analyzer::get_imports(&root_ast);

    if root_imports.is_empty() {
      let justfile = Analyzer::analyze(&root_ast, &[])?;
      let compilation = Compilation::new(root_ast, justfile, root_src);
      return Ok(compilation);
    }

    if !self.unstable {
      return Err(Error::Unstable {
        message: "The !include directive is currently unstable.".into(),
      });
    }

    let imported_asts = self.load_imported_justfiles(path, root_imports)?;

    let justfile = Analyzer::analyze(&root_ast, &imported_asts)?;
    let compilation = Compilation::new(root_ast, justfile, root_src);
    Ok(compilation)
  }

  fn load_imported_justfiles<'src>(
    &'src self,
    root_path: &Path,
    root_imports: Vec<Import>,
  ) -> RunResult<Vec<Ast<'src>>> {
    let mut imported_asts = vec![];

    let mut seen: HashSet<PathBuf> = HashSet::new();
    seen.insert(Self::canonicalize_path(root_path, root_path)?);

    let mut queue: VecDeque<(PathBuf, Vec<Import>)> = VecDeque::new();
    queue.push_back((root_path.to_owned(), root_imports));

    while let Some((cur_path, imports)) = queue.pop_front() {
      for mut import in imports {
        let given_path = import.path();
        let canonical_path = Self::canonicalize_path(given_path, &cur_path)?;

        if seen.contains(&canonical_path) {
          return Err(Error::CircularInclude {
            current: cur_path,
            include: canonical_path,
          });
        }
        seen.insert(canonical_path.clone());

        let src = self.load_and_alloc(&canonical_path)?;
        let ast = Compiler::parse(src)?;
        queue.push_back((canonical_path.clone(), Analyzer::get_imports(&ast)));
        import.add_canonical_path(canonical_path);
        imported_asts.push(ast);
      }
    }
    Ok(imported_asts)
  }

  fn canonicalize_path<'src, 'a>(
    import_path: &'a Path,
    current_file_path: &'a Path,
  ) -> RunResult<'src, PathBuf> {
    let canonical_path = if import_path.is_relative() {
      let current_dir = current_file_path.parent().ok_or(Error::Internal {
        message: format!(
          "Justfile path `{}` has no parent directory",
          import_path.display()
        ),
      })?;
      current_dir.join(import_path)
    } else {
      import_path.to_owned()
    };
    Ok(canonical_path.lexiclean())
  }

  fn load_and_alloc<'src>(&'src self, path: &Path) -> RunResult<&'src mut String> {
    let src = Self::load_file(path)?;
    Ok(self.arena.alloc(src))
  }

  fn load_file<'a>(path: &Path) -> RunResult<'a, String> {
    fs::read_to_string(path).map_err(|io_error| Error::Load {
      path: path.to_owned(),
      io_error,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::{Error, Lexiclean, Loader};
  use temptree::temptree;

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

    let loader = Loader::new(true);

    let justfile_a_path = tmp.path().join("justfile");
    let compilation = loader.load_and_compile(&justfile_a_path).unwrap();

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

    let loader = Loader::new(true);

    let justfile_a_path = tmp.path().join("justfile");
    let loader_output = loader.load_and_compile(&justfile_a_path).unwrap_err();

    assert_matches!(loader_output, Error::CircularInclude { current, include }
        if current == tmp.path().join("subdir").join("justfile_b").lexiclean() &&
        include == tmp.path().join("justfile").lexiclean()
    );
  }
}
