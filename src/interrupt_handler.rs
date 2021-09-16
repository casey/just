use crate::common::*;

pub(crate) struct InterruptHandler {
  blocks:      u32,
  interrupted: bool,
  verbosity:   Verbosity,
}

impl InterruptHandler {
  pub(crate) fn install(verbosity: Verbosity) -> Result<(), ctrlc::Error> {
    let mut instance = Self::instance();
    instance.verbosity = verbosity;
    ctrlc::set_handler(|| Self::instance().interrupt())
  }

  pub(crate) fn instance() -> MutexGuard<'static, Self> {
    lazy_static! {
      static ref INSTANCE: Mutex<InterruptHandler> = Mutex::new(InterruptHandler::new());
    }

    match INSTANCE.lock() {
      Ok(guard) => guard,
      Err(poison_error) => {
        eprintln!(
          "{}",
          Error::Internal {
            message: format!("interrupt handler mutex poisoned: {}", poison_error),
          }
          .color_display(Color::auto().stderr())
        );
        std::process::exit(EXIT_FAILURE);
      }
    }
  }

  fn new() -> Self {
    Self {
      blocks:      0,
      interrupted: false,
      verbosity:   Verbosity::default(),
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
      if self.verbosity.loud() {
        eprintln!(
          "{}",
          Error::Internal {
            message: "attempted to unblock interrupt handler, but handler was not blocked"
              .to_owned(),
          }
          .color_display(Color::auto().stderr())
        );
      }
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
