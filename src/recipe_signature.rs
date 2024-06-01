use super::*;

pub(crate) struct RecipeSignature<'a> {
  pub(crate) name: &'a str,
  pub(crate) recipe: &'a Recipe<'a>,
  pub(crate) is_default: bool,
}

impl ColorDisplay for &str {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    write!(f, "{}", color.default_recipe_name().paint(self))
  }
}

impl<'a> ColorDisplay for RecipeSignature<'a> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    if self.is_default {
      write!(f, "{}", self.name.color_display(color))?;
    } else {
      write!(f, "{}", self.name)?;
    }
    for parameter in &self.recipe.parameters {
      write!(f, " {}", parameter.color_display(color))?;
    }
    Ok(())
  }
}
