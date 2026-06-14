use super::*;

#[derive(Copy, Clone, Debug)]
pub(crate) enum ListFeature {
  ComparisonOperator,
  ListLiteral,
  LogicalOperator,
  NegationOperator,
  NonComparisonCondition,
}

impl ListFeature {
  pub(crate) fn error_kind(self) -> CompileErrorKind<'static> {
    match self {
      Self::ComparisonOperator => CompileErrorKind::ComparisonOperatorWithoutListsSetting,
      Self::ListLiteral => CompileErrorKind::ListLiteralWithoutListsSetting,
      Self::LogicalOperator => CompileErrorKind::LogicalOperatorWithoutListsSetting,
      Self::NegationOperator => CompileErrorKind::NegationOperatorWithoutListsSetting,
      Self::NonComparisonCondition => CompileErrorKind::NonComparisonConditionWithoutListsSetting,
    }
  }
}
