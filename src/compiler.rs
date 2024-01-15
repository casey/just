use {
  super::*,
  shellexpand::full,
};

pub(crate) struct Compiler;

impl Compiler {
  pub(crate) fn compile<'src>(
    unstable: bool,
    loader: &'src Loader,
    root: &Path,
  ) -> RunResult<'src, Compilation<'src>> {
    let mut asts: HashMap<PathBuf, Ast> = HashMap::new();
    let mut paths: HashMap<PathBuf, PathBuf> = HashMap::new();
    let mut srcs: HashMap<PathBuf, &str> = HashMap::new();
    let mut loaded = Vec::new();

    let mut stack = Vec::new();
    stack.push(Source::root(root));

    while let Some(current) = stack.pop() {
      let (relative, src) = loader.load(root, &current.path)?;
      loaded.push(relative.into());
      let tokens = Lexer::lex(relative, src)?;
      let mut ast = Parser::parse(
        &current.path,
        &current.namepath,
        current.depth,
        &tokens,
        &current.working_directory,
      )?;

      paths.insert(current.path.clone(), relative.into());
      srcs.insert(current.path.clone(), src);

      for item in &mut ast.items {
        match item {
          Item::Module {
            absolute,
            name,
            optional,
            relative,
          } => {
            if !unstable {
              return Err(Error::Unstable {
                message: "Modules are currently unstable.".into(),
              });
            }

            let parent = current.path.parent().unwrap();

            let import = if let Some(relative) = relative {
              let path = parent.join(Self::expand_tilde(&relative.cooked)?);

              if path.is_file() {
                Some(path)
              } else {
                None
              }
            } else {
              Self::find_module_file(parent, *name)?
            };

            if let Some(import) = import {
              if srcs.contains_key(&import) {
                return Err(Error::CircularImport {
                  current: current.path,
                  import,
                });
              }
              *absolute = Some(import.clone());
              stack.push(current.module(*name, import));
            } else if !*optional {
              return Err(Error::MissingModuleFile { module: *name });
            }
          }
          Item::Import {
            relative,
            absolute,
            optional,
            path,
          } => {
            let expanded_path = full(&relative.cooked).map_err(|error| Error::UnresolvedVariableInImport { variable: error.var_name, path: *path })?;

            let import = current
              .path
              .parent()
              .unwrap()
              .join(&*expanded_path)
              .lexiclean();

            if import.is_file() {
              if srcs.contains_key(&import) {
                return Err(Error::CircularImport {
                  current: current.path,
                  import,
                });
              }
              *absolute = Some(import.clone());
              stack.push(current.import(import));
            } else if !*optional {
              return Err(Error::MissingImportFile { path: *path });
            }
          }
          _ => {}
        }
      }

      asts.insert(current.path, ast.clone());
    }

    let justfile = Analyzer::analyze(&loaded, &paths, &asts, root)?;

    Ok(Compilation {
      asts,
      srcs,
      justfile,
      root: root.into(),
    })
  }

  fn find_module_file<'src>(parent: &Path, module: Name<'src>) -> RunResult<'src, Option<PathBuf>> {
    let mut candidates = vec![format!("{module}.just"), format!("{module}/mod.just")]
      .into_iter()
      .filter(|path| parent.join(path).is_file())
      .collect::<Vec<String>>();

    let directory = parent.join(module.lexeme());

    if directory.exists() {
      let entries = fs::read_dir(&directory).map_err(|io_error| SearchError::Io {
        io_error,
        directory: directory.clone(),
      })?;

      for entry in entries {
        let entry = entry.map_err(|io_error| SearchError::Io {
          io_error,
          directory: directory.clone(),
        })?;

        if let Some(name) = entry.file_name().to_str() {
          for justfile_name in search::JUSTFILE_NAMES {
            if name.eq_ignore_ascii_case(justfile_name) {
              candidates.push(format!("{module}/{name}"));
            }
          }
        }
      }
    }

    match candidates.as_slice() {
      [] => Ok(None),
      [file] => Ok(Some(parent.join(file).lexiclean())),
      found => Err(Error::AmbiguousModuleFile {
        found: found.into(),
        module,
      }),
    }
  }

  fn expand_tilde(path: &str) -> RunResult<'static, PathBuf> {
    Ok(if let Some(path) = path.strip_prefix("~/") {
      dirs::home_dir()
        .ok_or(Error::Homedir)?
        .join(path.trim_start_matches('/'))
    } else {
      PathBuf::from(path)
    })
  }

  #[cfg(test)]
  pub(crate) fn test_compile(src: &str) -> CompileResult<Justfile> {
    let tokens = Lexer::test_lex(src)?;
    let ast = Parser::parse(
      &PathBuf::new(),
      &Namepath::default(),
      0,
      &tokens,
      &PathBuf::new(),
    )?;
    let root = PathBuf::from("justfile");
    let mut asts: HashMap<PathBuf, Ast> = HashMap::new();
    asts.insert(root.clone(), ast);
    let mut paths: HashMap<PathBuf, PathBuf> = HashMap::new();
    paths.insert(root.clone(), root.clone());
    Analyzer::analyze(&[], &paths, &asts, &root)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, temptree::temptree};

  #[test]
  fn include_justfile() {
    let justfile_a = r#"
# A comment at the top of the file
import "./justfile_b"

#some_recipe: recipe_b
some_recipe:
    echo "some recipe"
"#;

    let justfile_b = r#"import "./subdir/justfile_c"

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
    let compilation = Compiler::compile(false, &loader, &justfile_a_path).unwrap();

    assert_eq!(compilation.root_src(), justfile_a);
  }

  #[test]
  fn recursive_includes_fail() {
    let justfile_a = r#"
# A comment at the top of the file
import "./subdir/justfile_b"

some_recipe: recipe_b
    echo "some recipe"

"#;

    let justfile_b = r#"
import "../justfile"

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
    let loader_output = Compiler::compile(false, &loader, &justfile_a_path).unwrap_err();

    assert_matches!(loader_output, Error::CircularImport { current, import }
        if current == tmp.path().join("subdir").join("justfile_b").lexiclean() &&
        import == tmp.path().join("justfile").lexiclean()
    );
  }
}
