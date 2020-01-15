use crate::common::*;

pub(crate) struct InterruptGuard;

impl InterruptGuard {
  pub(crate) fn new() -> Self {
    InterruptHandler::instance().block();
    Self
  }
}

impl Drop for InterruptGuard {
  fn drop(&mut self) {
    InterruptHandler::instance().unblock();
  }
}
