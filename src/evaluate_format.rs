use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
pub(crate) enum EvaluateFormat {
  #[default]
  Just,
  Shell,
}
