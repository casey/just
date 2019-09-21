use crate::common::*;

pub(crate) struct InterruptGuard;

impl InterruptGuard {
  pub(crate) fn new() -> InterruptGuard {
    InterruptHandler::instance().block();
    InterruptGuard
  }
}

impl Drop for InterruptGuard {
  fn drop(&mut self) {
    InterruptHandler::instance().unblock();
  }
}
