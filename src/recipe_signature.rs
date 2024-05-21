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
