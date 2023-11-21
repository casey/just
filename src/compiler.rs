use super::*;

pub(crate) struct Compiler;

impl Compiler {
  pub(crate) fn compile<'src>(
    unstable: bool,
    loader: &'src Loader,
    root: &Path,
  ) -> RunResult<'src, Foo<'src>> {
    let mut srcs: HashMap<PathBuf, &str> = HashMap::new();
    let mut asts: HashMap<PathBuf, Ast> = HashMap::new();

    let mut paths: Vec<PathBuf> = Vec::new();
    paths.push(root.into());

    while let Some(current) = paths.pop() {
      let src = loader.load(&current)?;
      let tokens = Lexer::lex(src)?;
      let mut ast = Parser::parse(&tokens)?;

      srcs.insert(current.clone(), src);

      for item in &mut ast.items {
        if let Item::Include { relative, absolute } = item {
          if !unstable {
            return Err(Error::Unstable {
              message: "The !include directive is currently unstable.".into(),
            });
          }

          let include = current.parent().unwrap().join(relative).lexiclean();

          if srcs.contains_key(&include) {
            return Err(Error::CircularInclude { current, include });
          }

          *absolute = Some(include.clone());

          paths.push(include);
        }
      }

      asts.insert(current.clone(), ast.clone());
    }

    let justfile = Analyzer::analyze(&asts, root)?;

    Ok(Foo {
      asts,
      srcs,
      justfile,
      root: root.into(),
    })
  }

  #[cfg(test)]
  pub(crate) fn test_compile(src: &str) -> CompileResult<Justfile> {
    let tokens = Lexer::lex(src)?;
    let ast = Parser::parse(&tokens)?;
    let root = PathBuf::from("<ROOT>");
    let mut asts: HashMap<PathBuf, Ast> = HashMap::new();
    asts.insert(root.clone(), ast);
    Analyzer::analyze(&asts, &root)
  }
}

#[derive(Debug)]
pub(crate) struct Foo<'src> {
  asts: HashMap<PathBuf, Ast<'src>>,
  justfile: Justfile<'src>,
  root: PathBuf,
  srcs: HashMap<PathBuf, &'src str>,
}

impl<'src> Foo<'src> {
  pub(crate) fn justfile(&self) -> &Justfile<'src> {
    &self.justfile
  }

  pub(crate) fn ast(&self) -> &Ast<'src> {
    self.asts.get(&self.root).unwrap()
  }

  pub(crate) fn src(&self) -> &'src str {
    self.srcs.get(&self.root).unwrap()
  }
}
