use crate::common::*;

#[derive(Clone, Debug)]
pub(crate) struct Closure<'src> {
  pub(crate) params: Vec<Name<'src>>,
  pub(crate) rule: Expression<'src>,
}

pub(crate) type NamedClosure<'src> = Binding<'src, Closure<'src>>;

fn join_name_seq(names: &[Name], sep: &str) -> String {
  // stabilization of the join trait is on the wishlist here
  names
    .iter()
    .map(Name::to_string)
    .collect::<Vec<_>>()
    .join(sep)
}

impl<'src> Display for NamedClosure<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    write!(
      f,
      "{}({}) := {}",
      self.name,
      join_name_seq(&self.value.params, ", "),
      self.value.rule
    )
  }
}
