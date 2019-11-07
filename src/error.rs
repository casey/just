use crate::common::*;

pub(crate) trait Error: Display {
  fn code(&self) -> i32 {
    EXIT_FAILURE
  }
}
