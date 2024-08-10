use super::*;

#[derive(Debug)]
pub(crate) struct Scope<'src: 'run, 'run> {
  parent: Option<&'run Self>,
  bindings: Table<'src, Binding<'src, String>>,
}

impl<'src, 'run> Scope<'src, 'run> {
  pub(crate) fn child(&'run self) -> Self {
    Self {
      parent: Some(self),
      bindings: Table::new(),
    }
  }

  pub(crate) fn root() -> Self {
    let mut root = Self {
      parent: None,
      bindings: Table::new(),
    };

    for (key, value) in constants() {
      root.bind(
        false,
        Name {
          token: Token {
            column: 0,
            kind: TokenKind::Identifier,
            length: key.len(),
            line: 0,
            offset: 0,
            path: Path::new("PRELUDE"),
            src: key,
          },
        },
        false,
        (*value).into(),
      );
    }

    root
  }

  pub(crate) fn bind(&mut self, export: bool, name: Name<'src>, private: bool, value: String) {
    self.bindings.insert(Binding {
      export,
      file_depth: 0,
      name,
      private,
      value,
    });
  }

  pub(crate) fn bound(&self, name: &str) -> bool {
    self.bindings.contains_key(name)
  }

  pub(crate) fn value(&self, name: &str) -> Option<&str> {
    if let Some(binding) = self.bindings.get(name) {
      Some(binding.value.as_ref())
    } else {
      self.parent?.value(name)
    }
  }

  pub(crate) fn bindings(&self) -> impl Iterator<Item = &Binding<String>> {
    self.bindings.values()
  }

  pub(crate) fn names(&self) -> impl Iterator<Item = &str> {
    self.bindings.keys().copied()
  }

  pub(crate) fn parent(&self) -> Option<&'run Self> {
    self.parent
  }
}
