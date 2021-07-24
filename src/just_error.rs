use crate::common::*;

pub(crate) enum JustError {
  Search(SearchError),
  Code(i32),
}

// TODO: remove this impl
impl From<i32> for JustError {
  fn from(code: i32) -> Self {
    Self::Code(code)
  }
}

impl From<SearchError> for JustError {
  fn from(error: SearchError) -> Self {
    Self::Search(error)
  }
}
