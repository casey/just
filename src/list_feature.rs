use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ListFeature {
  ComparisonOperator,
  ListLiteral,
  LogicalOperator,
  NonComparisonCondition,
}

impl ListFeature {
  pub(crate) fn error_kind<'src>(self) -> CompileErrorKind<'src> {
    match self {
      Self::ComparisonOperator => CompileErrorKind::ComparisonOperatorWithoutListsSetting,
      Self::ListLiteral => CompileErrorKind::ListLiteralWithoutListsSetting,
      Self::LogicalOperator => CompileErrorKind::LogicalOperatorWithoutListsSetting,
      Self::NonComparisonCondition => CompileErrorKind::NonComparisonConditionWithoutListsSetting,
    }
  }
}
