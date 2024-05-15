use super::*;

#[derive(Debug)]
pub(crate) struct Scope<'src: 'run, 'run> {
  parent: Option<&'run Scope<'src, 'run>>,
  bindings: Table<'src, Binding<'src, String>>,
}

impl<'src, 'run> Scope<'src, 'run> {
  pub(crate) fn child(&'run self) -> Scope<'src, 'run> {
    Self {
      parent: Some(self),
      bindings: Table::new(),
    }
  }

  pub(crate) fn new() -> Scope<'src, 'run> {
    Self {
      parent: None,
      bindings: Table::new(),
    }
  }

  pub(crate) fn bind(&mut self, export: bool, name: Name<'src>, value: String) {
    self.bindings.insert(Binding {
      depth: 0,
      export,
      name,
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

  pub(crate) fn parent(&self) -> Option<&'run Scope<'src, 'run>> {
    self.parent
  }
}
