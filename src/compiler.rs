use super::*;

pub(crate) struct Compiler;

impl Compiler {
  pub(crate) fn compile<'src>(
    loader: &'src Loader,
    root: &Path,
    unstable: bool,
  ) -> RunResult<'src, Compilation<'src>> {
    let mut asts = HashMap::<PathBuf, Ast>::new();
    let mut loaded = Vec::new();
    let mut paths = HashMap::<PathBuf, PathBuf>::new();
    let mut srcs = HashMap::<PathBuf, &str>::new();

    let mut stack = Vec::new();
    stack.push(Source::root(root));

    while let Some(current) = stack.pop() {
      let (relative, src) = loader.load(root, &current.path)?;
      loaded.push(relative.into());
      let tokens = Lexer::lex(relative, src)?;
      let mut ast = Parser::parse(
        current.file_depth,
        &current.path,
        &current.import_offsets,
        &current.namepath,
        &tokens,
        &current.working_directory,
      )?;

      paths.insert(current.path.clone(), relative.into());
      srcs.insert(current.path.clone(), src);

      let ast_unstable = ast.items.iter().any(|item| {
        matches!(
          item,
          Item::Set(Set {
            name: _,
            value: Setting::Unstable(true)
          })
        )
      });

      for item in &mut ast.items {
        match item {
          Item::Module {
            absolute,
            name,
            optional,
            relative,
            ..
          } => {
            let parent = current.path.parent().unwrap();

            let relative = relative
              .as_ref()
              .map(|relative| Self::expand_tilde(&relative.cooked))
              .transpose()?;

            let import = Self::find_module_file(parent, *name, relative.as_deref())?;

            if let Some(import) = import {
              if current.file_path.contains(&import) {
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
            absolute_paths,
            optional,
            path,
          } => {
            let import = current
              .path
              .parent()
              .unwrap()
              .join(Self::expand_tilde(&relative.cooked)?)
              .lexiclean();

            let has_glob = import.as_os_str().as_encoded_bytes().contains(&b'*');

            if has_glob {
              if !(unstable || ast_unstable) {
                return Err(Error::UnstableFeature {
                  unstable_feature: UnstableFeature::ImportPathGlob,
                });
              }

              let import_base_str = import
                .to_str()
                .ok_or_else(|| Error::ImportGlobUnicode { path: *path })?;
              let glob_options = glob::MatchOptions {
                case_sensitive: true,
                require_literal_separator: true,
                require_literal_leading_dot: false,
              };
              let import_paths =
                glob::glob_with(import_base_str, glob_options).map_err(|pattern_err| {
                  Error::ImportGlobPattern {
                    pattern_err,
                    path: *path,
                  }
                })?;
              for import in import_paths {
                let import = import.map_err(|glob_err| Error::ImportGlobIteration {
                  path: *path,
                  glob_err,
                })?;

                if import.is_file() {
                  if current.file_path.contains(&import) {
                    return Err(Error::CircularImport {
                      current: current.path,
                      import,
                    });
                  }
                  absolute_paths.push(import.clone());
                  stack.push(current.import(import, path.offset));
                } else if import.is_dir() {
                  return Err(Error::ImportGlobDirectory {
                    import,
                    path: *path,
                  });
                }
              }
            } else if import.is_file() {
              if current.file_path.contains(&import) {
                return Err(Error::CircularImport {
                  current: current.path,
                  import,
                });
              }
              absolute_paths.push(import.clone());
              stack.push(current.import(import, path.offset));
            } else if !*optional {
              return Err(Error::MissingImportFile { path: *path });
            }
          }
          _ => {}
        }
      }

      asts.insert(current.path, ast.clone());
    }

    let justfile = Analyzer::analyze(&asts, None, &[], &loaded, None, &paths, root)?;

    Ok(Compilation {
      asts,
      justfile,
      root: root.into(),
      srcs,
    })
  }

  fn find_module_file<'src>(
    parent: &Path,
    module: Name<'src>,
    path: Option<&Path>,
  ) -> RunResult<'src, Option<PathBuf>> {
    let mut candidates = Vec::new();

    if let Some(path) = path {
      let full = parent.join(path);

      if full.is_file() {
        return Ok(Some(full));
      }

      candidates.push((path.join("mod.just"), true));

      for name in search::JUSTFILE_NAMES {
        candidates.push((path.join(name), false));
      }
    } else {
      candidates.push((format!("{module}.just").into(), true));
      candidates.push((format!("{module}/mod.just").into(), true));

      for name in search::JUSTFILE_NAMES {
        candidates.push((format!("{module}/{name}").into(), false));
      }
    }

    let mut grouped = BTreeMap::<PathBuf, Vec<(PathBuf, bool)>>::new();

    for (candidate, case_sensitive) in candidates {
      let candidate = parent.join(candidate).lexiclean();
      grouped
        .entry(candidate.parent().unwrap().into())
        .or_default()
        .push((candidate, case_sensitive));
    }

    let mut found = Vec::new();

    for (directory, candidates) in grouped {
      let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(io_error) => {
          if io_error.kind() == io::ErrorKind::NotFound {
            continue;
          }

          return Err(
            SearchError::Io {
              io_error,
              directory,
            }
            .into(),
          );
        }
      };

      for entry in entries {
        let entry = entry.map_err(|io_error| SearchError::Io {
          io_error,
          directory: directory.clone(),
        })?;

        if let Some(name) = entry.file_name().to_str() {
          for (candidate, case_sensitive) in &candidates {
            let candidate_name = candidate.file_name().unwrap().to_str().unwrap();

            let eq = if *case_sensitive {
              name == candidate_name
            } else {
              name.eq_ignore_ascii_case(candidate_name)
            };

            if eq {
              found.push(candidate.parent().unwrap().join(name));
            }
          }
        }
      }
    }

    if found.len() > 1 {
      found.sort();
      Err(Error::AmbiguousModuleFile {
        found: found
          .into_iter()
          .map(|found| found.strip_prefix(parent).unwrap().into())
          .collect(),
        module,
      })
    } else {
      Ok(found.into_iter().next())
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
      0,
      &PathBuf::new(),
      &[],
      &Namepath::default(),
      &tokens,
      &PathBuf::new(),
    )?;
    let root = PathBuf::from("justfile");
    let mut asts: HashMap<PathBuf, Ast> = HashMap::new();
    asts.insert(root.clone(), ast);
    let mut paths: HashMap<PathBuf, PathBuf> = HashMap::new();
    paths.insert(root.clone(), root.clone());
    Analyzer::analyze(&asts, None, &[], &[], None, &paths, &root)
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
    let compilation = Compiler::compile(&loader, &justfile_a_path, false).unwrap();

    assert_eq!(compilation.root_src(), justfile_a);
  }

  #[test]
  fn recursive_includes_fail() {
    let tmp = temptree! {
      justfile: "import './subdir/b'\na: b",
      subdir: {
        b: "import '../justfile'\nb:"
      }
    };

    let loader = Loader::new();

    let justfile_a_path = tmp.path().join("justfile");
    let loader_output = Compiler::compile(&loader, &justfile_a_path, false).unwrap_err();

    assert_matches!(loader_output, Error::CircularImport { current, import }
      if current == tmp.path().join("subdir").join("b").lexiclean() &&
      import == tmp.path().join("justfile").lexiclean()
    );
  }

  #[test]
  fn glob_imports() {
    let justfile_top = r#"
import "./subdir/*.just"
          "#;
    let justfile_a = r"
a:
    ";
    let justfile_b = r"
b:
             ";
    let unused_justfile = r"
x:
             ";
    let tmp = temptree! {
      justfile: justfile_top,
      subdir: {
        "a.just": justfile_a,
        "b.just": justfile_b,
        unusued: unused_justfile,
      }
    };
    let loader = Loader::new();
    let justfile_path = tmp.path().join("justfile");
    let compilation = Compiler::compile(&loader, &justfile_path, true).unwrap();
    let imported_files: HashSet<&PathBuf> = compilation.srcs.keys().collect();
    assert_eq!(
      imported_files,
      [
        justfile_path,
        tmp.path().join("subdir/a.just"),
        tmp.path().join("subdir/b.just")
      ]
      .iter()
      .collect()
    );
  }

  #[test]
  fn malformed_glob_imports() {
    let justfile = r#"
import "./subdir/****.just"
    "#;
    let tmp = temptree! {
        justfile: justfile,
    };
    let loader = Loader::new();
    let justfile_path = tmp.path().join("justfile");
    let error = Compiler::compile(&loader, &justfile_path, true).unwrap_err();
    let expected = "error: Invalid glob pattern: Pattern syntax error";
    let actual = error.color_display(Color::never()).to_string();
    assert!(actual.contains(expected), "Actual: {actual}");
  }

  #[test]
  fn glob_import_match_dir_errors() {
    let justfile_top = r#"
import "./subdir/*"
          "#;
    let justfile_a = r"
a:
    ";
    let tmp = temptree! {
      justfile: justfile_top,
      subdir: {
        "a.just": justfile_a,
        subsubdir: {

        }
      }
    };
    let loader = Loader::new();
    let justfile_path = tmp.path().join("justfile");
    let error = Compiler::compile(&loader, &justfile_path, true).unwrap_err();
    let expected = "error: A glob import cannot match a directory";
    let actual = error.color_display(Color::never()).to_string();
    assert!(actual.contains(expected), "Actual: {actual}");
  }

  #[test]
  fn find_module_file() {
    #[track_caller]
    fn case(path: Option<&str>, files: &[&str], expected: Result<Option<&str>, &[&str]>) {
      let module = Name {
        token: Token {
          column: 0,
          kind: TokenKind::Identifier,
          length: 3,
          line: 0,
          offset: 0,
          path: Path::new(""),
          src: "foo",
        },
      };

      let tempdir = tempfile::tempdir().unwrap();

      for file in files {
        if let Some(parent) = Path::new(file).parent() {
          fs::create_dir_all(tempdir.path().join(parent)).unwrap();
        }

        fs::write(tempdir.path().join(file), "").unwrap();
      }

      let actual = Compiler::find_module_file(tempdir.path(), module, path.map(Path::new));

      match expected {
        Err(expected) => match actual.unwrap_err() {
          Error::AmbiguousModuleFile { found, .. } => {
            assert_eq!(
              found,
              expected
                .iter()
                .map(|expected| expected.replace('/', std::path::MAIN_SEPARATOR_STR).into())
                .collect::<Vec<PathBuf>>()
            );
          }
          _ => panic!("unexpected error"),
        },
        Ok(Some(expected)) => assert_eq!(
          actual.unwrap().unwrap(),
          tempdir
            .path()
            .join(expected.replace('/', std::path::MAIN_SEPARATOR_STR))
        ),
        Ok(None) => assert_eq!(actual.unwrap(), None),
      }
    }

    case(None, &["foo.just"], Ok(Some("foo.just")));
    case(None, &["FOO.just"], Ok(None));
    case(None, &["foo/mod.just"], Ok(Some("foo/mod.just")));
    case(None, &["foo/MOD.just"], Ok(None));
    case(None, &["foo/justfile"], Ok(Some("foo/justfile")));
    case(None, &["foo/JUSTFILE"], Ok(Some("foo/JUSTFILE")));
    case(None, &["foo/.justfile"], Ok(Some("foo/.justfile")));
    case(None, &["foo/.JUSTFILE"], Ok(Some("foo/.JUSTFILE")));
    case(
      None,
      &["foo/.justfile", "foo/justfile"],
      Err(&["foo/.justfile", "foo/justfile"]),
    );
    case(None, &["foo/JUSTFILE"], Ok(Some("foo/JUSTFILE")));

    case(Some("bar"), &["bar"], Ok(Some("bar")));
    case(Some("bar"), &["bar/mod.just"], Ok(Some("bar/mod.just")));
    case(Some("bar"), &["bar/justfile"], Ok(Some("bar/justfile")));
    case(Some("bar"), &["bar/JUSTFILE"], Ok(Some("bar/JUSTFILE")));
    case(Some("bar"), &["bar/.justfile"], Ok(Some("bar/.justfile")));
    case(Some("bar"), &["bar/.JUSTFILE"], Ok(Some("bar/.JUSTFILE")));

    case(
      Some("bar"),
      &["bar/justfile", "bar/mod.just"],
      Err(&["bar/justfile", "bar/mod.just"]),
    );
  }
}
