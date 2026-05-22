use super::*;

pub(crate) struct RecipeSignature<'a> {
  pub(crate) name: &'a str,
  pub(crate) recipe: &'a Recipe<'a>,
}

impl ColorDisplay for RecipeSignature<'_> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    write!(f, "{}", self.name)?;
    if self.recipe.parameters.iter().any(Parameter::is_option) {
      write!(f, " {}", color.argument().paint("[OPTIONS]"))?;
    }
    for parameter in &self.recipe.parameters {
      if parameter.is_option() {
        continue;
      }
      write!(f, " {}", parameter.color_display(color))?;
    }
    Ok(())
  }
}
