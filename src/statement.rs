use super::*;

#[derive(PartialEq, Debug, Clone, Serialize)]
pub(crate) struct Statement<'src> {
  pub(crate) words: Vec<Word<'src>>,
}
