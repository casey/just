use super::*;

// todo:
// - test that functions parse:
//   - 0, 1, 2 parameters
// - resolve undefined variables in function body
// - allow function parameters in function body
// - parse function calls
// - evaluate function calls, binding arguments to parameters
// - test that functions dump correctly
// - make functions unstable
// - catch function stack overflow
// - allow recursion in functions

#[derive(Debug, Clone)]
pub(crate) struct FunctionDefinition<'src> {
  pub(crate) name: Name<'src>,
  pub(crate) parameters: Vec<Name<'src>>,
  pub(crate) body: Expression<'src>,
}

impl<'src> Keyed<'src> for FunctionDefinition<'src> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}

impl<'src> Display for FunctionDefinition<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    write!(f, "{}(", self.name)?;

    for (i, parameter) in self.parameters.iter().enumerate() {
      if i > 0 {
        write!(f, ", ")?;
      }

      write!(f, "{parameter}")?;
    }

    write!(f, ") := {}", self.body)?;

    Ok(())
  }
}
