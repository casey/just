use super::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum ListFeature {
  ArgMax,
  BoolFunction,
  ComparisonOperator,
  Flag,
  IfWithoutElse,
  JoinListFunction,
  ListConcatenationOperator,
  ListLiteral,
  LogicalOperator,
  Multiple,
  NegationOperator,
  NonComparisonCondition,
  ShowFunction,
  SplitFunction,
  WhichFunction,
}

impl ListFeature {
  pub(crate) fn function(self) -> bool {
    match self {
      Self::BoolFunction
      | Self::JoinListFunction
      | Self::ShowFunction
      | Self::SplitFunction
      | Self::WhichFunction => true,
      Self::ArgMax
      | Self::ComparisonOperator
      | Self::Flag
      | Self::IfWithoutElse
      | Self::ListConcatenationOperator
      | Self::ListLiteral
      | Self::LogicalOperator
      | Self::Multiple
      | Self::NegationOperator
      | Self::NonComparisonCondition => false,
    }
  }
}

impl Display for ListFeature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::ArgMax => write!(f, "`[arg(max)]` requires `set lists`"),
      Self::BoolFunction => write!(f, "the `bool()` function requires `set lists`"),
      Self::ComparisonOperator => write!(f, "comparison operators require `set lists`"),
      Self::Flag => write!(f, "`flag` arguments require `set lists`"),
      Self::IfWithoutElse => write!(f, "`if` without `else` requires `set lists`"),
      Self::JoinListFunction => write!(f, "the `join_list()` function requires `set lists`"),
      Self::ListConcatenationOperator => {
        write!(f, "list concatenation operator `++` requires `set lists`")
      }
      Self::ListLiteral => write!(f, "list literals require `set lists`"),
      Self::LogicalOperator => write!(f, "logical operators require `set lists`"),
      Self::Multiple => write!(f, "`[arg(multiple)]` requires `set lists`"),
      Self::NegationOperator => write!(f, "negation operator requires `set lists`"),
      Self::NonComparisonCondition => write!(
        f,
        "`if` and `assert` conditions other than comparisons require `set lists`"
      ),
      Self::ShowFunction => write!(f, "the `show()` function requires `set lists`"),
      Self::SplitFunction => write!(f, "the `split()` function requires `set lists`"),
      Self::WhichFunction => write!(f, "the `which()` function requires `set lists`"),
    }
  }
}
