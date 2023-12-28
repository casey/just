use super::*;

pub(crate) struct Compiler;

fn find_module<'src>(parent: &Path, module: Name<'src>) -> RunResult<'src, PathBuf> {
  let modules = vec![
    format!("{module}.just"),
    format!("{module}/mod.just"),
    format!("{module}/justfile"),
  ]
  .into_iter()
  .filter(|path| parent.join(path).is_file())
  .collect::<Vec<String>>();

  match modules.as_slice() {
    [] => Err(Error::MissingModuleFile { module }),
    [file] => Ok(parent.join(file).lexiclean()),
    found => Err(Error::AmbiguousModuleFile {
      found: found.into(),
      module,
    }),
  }
}

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

    let mut stack: Vec<PathBuf> = Vec::new();
    stack.push(root.into());

    while let Some(current) = stack.pop() {
      let (relative, src) = loader.load(root, &current)?;
      loaded.push(relative.into());
      let tokens = Lexer::lex(relative, src)?;
      let mut ast = Parser::parse(&tokens)?;

      paths.insert(current.clone(), relative.into());
      srcs.insert(current.clone(), src);

      for item in &mut ast.items {
        match item {
          Item::Mod { name, absolute } => {
            if !unstable {
              return Err(Error::Unstable {
                message: "Modules are currently unstable.".into(),
              });
            }

            let parent = current.parent().unwrap();

            let import = find_module(parent, *name)?;

            if srcs.contains_key(&import) {
              return Err(Error::CircularImport { current, import });
            }
            *absolute = Some(import.clone());
            stack.push(import);
          }
          Item::Import { relative, absolute } => {
            let import = current.parent().unwrap().join(&relative.cooked).lexiclean();
            if srcs.contains_key(&import) {
              return Err(Error::CircularImport { current, import });
            }
            *absolute = Some(import.clone());
            stack.push(import);
          }
          _ => {}
        }
      }

      asts.insert(current.clone(), ast.clone());
    }

    let justfile = Analyzer::analyze(&loaded, &paths, &asts, root)?;

    Ok(Compilation {
      asts,
      srcs,
      justfile,
      root: root.into(),
    })
  }

  #[cfg(test)]
  pub(crate) fn test_compile(src: &str) -> CompileResult<Justfile> {
    let tokens = Lexer::test_lex(src)?;
    let ast = Parser::parse(&tokens)?;
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
