use crate::common::*;

pub struct InterruptHandler {
  blocks: u32,
  interrupted: bool,
}

impl InterruptHandler {
  pub fn install() -> Result<(), ctrlc::Error> {
    ctrlc::set_handler(|| InterruptHandler::instance().interrupt())
  }

  pub fn instance() -> MutexGuard<'static, InterruptHandler> {
    lazy_static! {
      static ref INSTANCE: Mutex<InterruptHandler> = Mutex::new(InterruptHandler::new());
    }

    match INSTANCE.lock() {
      Ok(guard) => guard,
      Err(poison_error) => die!(
        "{}",
        RuntimeError::Internal {
          message: format!("interrupt handler mutex poisoned: {}", poison_error),
        }
      ),
    }
  }

  fn new() -> InterruptHandler {
    InterruptHandler {
      blocks: 0,
      interrupted: false,
    }
  }

  fn interrupt(&mut self) {
    self.interrupted = true;

    if self.blocks > 0 {
      return;
    }

    Self::exit();
  }

  fn exit() {
    process::exit(130);
  }

  pub fn block(&mut self) {
    self.blocks += 1;
  }

  pub fn unblock(&mut self) {
    if self.blocks == 0 {
      die!(
        "{}",
        RuntimeError::Internal {
          message: "attempted to unblock interrupt handler, but handler was not blocked"
            .to_string(),
        }
      );
    }

    self.blocks -= 1;

    if self.interrupted {
      Self::exit();
    }
  }

  pub fn guard<T, F: FnOnce() -> T>(function: F) -> T {
    let _guard = InterruptGuard::new();
    function()
  }
}
