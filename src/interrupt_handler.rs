use crate::common::*;

pub(crate) struct InterruptHandler {
  blocks: u32,
  interrupted: bool,
}

impl InterruptHandler {
  pub(crate) fn install() -> Result<(), ctrlc::Error> {
    ctrlc::set_handler(|| InterruptHandler::instance().interrupt())
  }

  pub(crate) fn instance() -> MutexGuard<'static, InterruptHandler> {
    lazy_static! {
      static ref INSTANCE: Mutex<InterruptHandler> = Mutex::new(InterruptHandler::new());
    }

    match INSTANCE.lock() {
      Ok(guard) => guard,
      Err(poison_error) => {
        eprintln!(
          "{}",
          RuntimeError::Internal {
            message: format!("interrupt handler mutex poisoned: {}", poison_error),
          }
        );
        std::process::exit(EXIT_FAILURE);
      }
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

  pub(crate) fn block(&mut self) {
    self.blocks += 1;
  }

  pub(crate) fn unblock(&mut self) {
    if self.blocks == 0 {
      eprintln!(
        "{}",
        RuntimeError::Internal {
          message: "attempted to unblock interrupt handler, but handler was not blocked"
            .to_string(),
        }
      );
      std::process::exit(EXIT_FAILURE);
    }

    self.blocks -= 1;

    if self.interrupted {
      Self::exit();
    }
  }

  pub(crate) fn guard<T, F: FnOnce() -> T>(function: F) -> T {
    let _guard = InterruptGuard::new();
    function()
  }
}
