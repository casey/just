use super::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum ListFeature {
  BoolFunction,
  ComparisonOperator,
  Flag,
  IfWithoutElse,
  ListLiteral,
  LogicalOperator,
  NegationOperator,
  NonComparisonCondition,
  ShowFunction,
}

impl Display for ListFeature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::BoolFunction => write!(f, "the `bool()` function requires `set lists`"),
      Self::ComparisonOperator => write!(f, "comparison operators require `set lists`"),
      Self::Flag => write!(f, "`flag` arguments require `set lists`"),
      Self::IfWithoutElse => write!(f, "`if` without `else` requires `set lists`"),
      Self::ListLiteral => write!(f, "list literals require `set lists`"),
      Self::LogicalOperator => write!(f, "logical operators require `set lists`"),
      Self::NegationOperator => write!(f, "negation operator requires `set lists`"),
      Self::NonComparisonCondition => write!(
        f,
        "`if` and `assert` conditions other than comparisons require `set lists`"
      ),
      Self::ShowFunction => write!(f, "the `show()` function requires `set lists`"),
    }
  }
}
