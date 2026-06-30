use super::*;

#[derive(Default, Debug, Clone)]
pub(crate) struct AttributeSet<'src>(BTreeMap<Attribute<'src>, Name<'src>>);

impl<'src> AttributeSet<'src> {
  pub(crate) fn len(&self) -> usize {
    self.0.len()
  }

  pub(crate) fn contains(&self, kind: AttributeKind) -> bool {
    self.0.keys().any(|attribute| attribute.kind() == kind)
  }

  pub(crate) fn get(&self, kind: AttributeKind) -> Option<&Attribute<'src>> {
    self.0.keys().find(|attribute| attribute.kind() == kind)
  }

  pub(crate) fn groups(&self) -> Vec<StringLiteral<'src>> {
    self
      .0
      .keys()
      .filter_map(|attribute| {
        if let Attribute::Group(group) = attribute {
          Some(group.clone())
        } else {
          None
        }
      })
      .collect()
  }

  pub(crate) fn name(&self, attribute: &Attribute<'src>) -> Name<'src> {
    self.0[attribute]
  }

  pub(crate) fn private(&self) -> bool {
    self.contains(AttributeKind::Private)
  }

  pub(crate) fn iter(&self) -> btree_map::Keys<'_, Attribute<'src>, Name<'src>> {
    self.0.keys()
  }

  pub(crate) fn into_items(self) -> btree_map::IntoIter<Attribute<'src>, Name<'src>> {
    self.0.into_iter()
  }

  pub(crate) fn ensure_valid_attributes(
    &self,
    item_kind: &'static str,
    item_token: Token<'src>,
    valid: &[AttributeKind],
  ) -> Result<(), CompileError<'src>> {
    for attribute in self.0.keys() {
      if !valid.contains(&attribute.kind()) {
        return Err(item_token.error(CompileErrorKind::InvalidAttribute {
          item_kind,
          item_name: item_token.lexeme(),
          attribute: Box::new(attribute.clone()),
        }));
      }
    }
    Ok(())
  }
}

impl PartialEq for AttributeSet<'_> {
  fn eq(&self, other: &Self) -> bool {
    self.0.keys().eq(other.0.keys())
  }
}

impl Serialize for AttributeSet<'_> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.collect_seq(self.0.keys())
  }
}

impl<'src> FromIterator<(Attribute<'src>, Name<'src>)> for AttributeSet<'src> {
  fn from_iter<T: IntoIterator<Item = (Attribute<'src>, Name<'src>)>>(iter: T) -> Self {
    Self(iter.into_iter().collect())
  }
}

impl<'src, 'a> IntoIterator for &'a AttributeSet<'src> {
  type Item = &'a Attribute<'src>;

  type IntoIter = btree_map::Keys<'a, Attribute<'src>, Name<'src>>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.keys()
  }
}

impl<'src> IntoIterator for AttributeSet<'src> {
  type Item = Attribute<'src>;

  type IntoIter = btree_map::IntoKeys<Attribute<'src>, Name<'src>>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_keys()
  }
}
