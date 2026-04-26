use {super::*, std::collections};

pub(crate) type EvaluatedAttributeSet<'src> = AttributeSet<'src, String>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct AttributeSet<'src, T = Expression<'src>>(BTreeSet<Attribute<'src, T>>);

impl<T> Default for AttributeSet<'_, T> {
  fn default() -> Self {
    Self(BTreeSet::new())
  }
}

impl<'src, T: Ord> AttributeSet<'src, T> {
  pub(crate) fn len(&self) -> usize {
    self.0.len()
  }

  pub(crate) fn contains(&self, target: AttributeDiscriminant) -> bool {
    self.0.iter().any(|attr| attr.discriminant() == target)
  }

  pub(crate) fn get(&self, discriminant: AttributeDiscriminant) -> Option<&Attribute<'src, T>> {
    self
      .0
      .iter()
      .find(|attr| discriminant == attr.discriminant())
  }

  pub(crate) fn iter<'a>(&'a self) -> collections::btree_set::Iter<'a, Attribute<'src, T>> {
    self.0.iter()
  }
}

impl<'src> AttributeSet<'src> {
  pub(crate) fn ensure_valid_attributes(
    &self,
    item_kind: &'static str,
    item_token: Token<'src>,
    valid: &[AttributeDiscriminant],
  ) -> Result<(), CompileError<'src>> {
    for attribute in &self.0 {
      let discriminant = attribute.discriminant();
      if !valid.contains(&discriminant) {
        return Err(item_token.error(CompileErrorKind::InvalidAttribute {
          item_kind,
          item_name: item_token.lexeme(),
          attribute: Box::new(attribute.clone()),
        }));
      }
    }
    Ok(())
  }

  pub(crate) fn evaluate(
    self,
    assignments: &Table<'src, Assignment<'src>>,
    overrides: &HashMap<Number, String>,
  ) -> RunResult<'src, EvaluatedAttributeSet<'src>> {
    self
      .0
      .into_iter()
      .map(|attribute| attribute.evaluate(assignments, overrides))
      .collect::<RunResult<BTreeSet<_>>>()
      .map(AttributeSet)
  }
}

impl<'src, T: Ord> FromIterator<Attribute<'src, T>> for AttributeSet<'src, T> {
  fn from_iter<I: IntoIterator<Item = attribute::Attribute<'src, T>>>(iter: I) -> Self {
    Self(iter.into_iter().collect())
  }
}

impl<'src, 'a, T> IntoIterator for &'a AttributeSet<'src, T> {
  type Item = &'a Attribute<'src, T>;

  type IntoIter = collections::btree_set::Iter<'a, Attribute<'src, T>>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.iter()
  }
}

impl<'src, T> IntoIterator for AttributeSet<'src, T> {
  type Item = Attribute<'src, T>;

  type IntoIter = collections::btree_set::IntoIter<Attribute<'src, T>>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
