use super::*;

pub(crate) struct RecipeSignature<'a> {
  pub(crate) default: bool,
  pub(crate) name: &'a str,
  pub(crate) recipe: &'a Recipe<'a>,
}

impl<'a> ColorDisplay for RecipeSignature<'a> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    if self.default {
      write!(f, "{}", color.default_recipe().paint(self.name))?;
    } else {
      write!(f, "{}", self.name)?;
    }
    for parameter in &self.recipe.parameters {
      write!(f, " {}", parameter.color_display(color))?;
    }
    Ok(())
  }
}
