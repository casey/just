use super::*;

pub(crate) struct RecipeSignature<'a> {
  pub(crate) name: &'a str,
  pub(crate) recipe: &'a Recipe<'a>,
}

impl<'a> ColorDisplay for RecipeSignature<'a> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    write!(f, "{}", self.name)?;
    for parameter in &self.recipe.parameters {
      write!(f, " {}", parameter.color_display(color))?;
    }
    Ok(())
  }
}

#[derive(Debug)]
pub(crate) struct SignatureWidths<'a> {
  pub(crate) widths: BTreeMap<&'a str, usize>,
  pub(crate) threshold: usize,
  pub(crate) max_width: usize,
}

impl<'a> SignatureWidths<'a> {
  pub fn empty() -> Self {
    Self {
      widths: BTreeMap::new(),
      threshold: 50,
      max_width: 0,
    }
  }

  pub fn add_string_custom_width(&mut self, string: &'a str, width: usize) {
    self.widths.insert(string, width);
    self.max_width = self.max_width.max(width).min(self.threshold);
  }

  pub fn add_entries<'outer>(&mut self, entries: &Vec<ListEntry<'a, 'outer>>) {
    for entry in entries {
      self.add_entry(entry);
    }
  }

  pub fn add_entry<'file>(&mut self, entry: &ListEntry<'a, 'file>) {
    if !entry.recipe.is_public() {
      return;
    }

    for name in iter::once(entry.recipe.name()).chain(entry.aliases.iter().copied()) {
      let format = if entry.prefix.is_empty() {
        Cow::Borrowed(name)
      } else {
        Cow::Owned(format!("{}{}", entry.prefix, name))
      };
      let width = UnicodeWidthStr::width(
        RecipeSignature {
          name: &format,
          recipe: entry.recipe,
        }
        .color_display(Color::never())
        .to_string()
        .as_str(),
      );
      self.widths.insert(name, width);
      if width <= self.threshold {
        self.max_width = self.max_width.max(width);
      }
    }
  }
}
