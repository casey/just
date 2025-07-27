use super::*;

pub(crate) struct RecipeSignature<'a> {
  pub(crate) name: &'a str,
  pub(crate) recipe: &'a Recipe<'a>,
}

impl ColorDisplay for RecipeSignature<'_> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    write!(f, "{}", self.name)?;
    for (_, parameter) in &self.recipe.flags {
      write!(f, " {}", parameter.color_display(color))?;
    }
    for parameter in &self.recipe.parameters {
      write!(f, " {}", parameter.color_display(color))?;
    }
    Ok(())
  }
}
