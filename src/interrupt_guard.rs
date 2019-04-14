use crate::common::*;

pub struct InterruptGuard;

impl InterruptGuard {
  pub fn new() -> InterruptGuard {
    InterruptHandler::instance().block();
    InterruptGuard
  }
}

impl Drop for InterruptGuard {
  fn drop(&mut self) {
    InterruptHandler::instance().unblock();
  }
}
