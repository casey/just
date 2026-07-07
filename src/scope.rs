use super::*;

#[derive(Debug)]
pub(crate) struct Scope<'src: 'run, 'run> {
  bindings: Table<'src, Binding<'src>>,
  parent: Option<&'run Self>,
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

    for (i, (key, value)) in constants().iter().enumerate() {
      root.bind(Binding {
        attributes: AttributeSet::new(),
        eager: false,
        export: false,
        file_depth: 0,
        name: Name {
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
        number: Numerator::constant(i),
        prelude: true,
        private: false,
        value: (*value).into(),
      });
    }

    root
  }

  pub(crate) fn bind(&mut self, binding: Binding<'src>) {
    self.bindings.insert(binding);
  }

  pub(crate) fn binding(&self, name: &str) -> Option<&Binding<'src>> {
    if let Some(binding) = self.bindings.get(name) {
      Some(binding)
    } else {
      self.parent?.binding(name)
    }
  }

  pub(crate) fn value(&self, name: &str) -> Option<&Value> {
    Some(&self.binding(name)?.value)
  }

  pub(crate) fn bindings(&self) -> impl Iterator<Item = &Binding<'src>> {
    self.bindings.values()
  }

  pub(crate) fn parent(&self) -> Option<&'run Self> {
    self.parent
  }
}
