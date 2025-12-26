use super::*;

pub(crate) struct SignalHandler {
  caught: Option<Signal>,
  children: BTreeMap<i32, Command>,
  initialized: bool,
  verbosity: Verbosity,
}

impl SignalHandler {
  pub(crate) fn install(verbosity: Verbosity) -> RunResult<'static> {
    let mut instance = Self::instance();
    instance.verbosity = verbosity;
    if !instance.initialized {
      Platform::install_signal_handler(|signal| Self::instance().handle(signal))?;
      instance.initialized = true;
    }
    Ok(())
  }

  pub(crate) fn instance() -> MutexGuard<'static, Self> {
    static INSTANCE: Mutex<SignalHandler> = Mutex::new(SignalHandler::new());

    match INSTANCE.lock() {
      Ok(guard) => guard,
      Err(poison_error) => {
        eprintln!(
          "{}",
          Error::Internal {
            message: format!("signal handler mutex poisoned: {poison_error}"),
          }
          .color_display(Color::auto().stderr())
        );
        process::exit(EXIT_FAILURE);
      }
    }
  }

  const fn new() -> Self {
    Self {
      caught: None,
      children: BTreeMap::new(),
      initialized: false,
      verbosity: Verbosity::default(),
    }
  }

  fn handle(&mut self, signal: Signal) {
    if signal.is_fatal() {
      if self.children.is_empty() {
        process::exit(signal.code());
      }

      if self.caught.is_none() {
        self.caught = Some(signal);
      }
    }

    match signal {
      // SIGHUP, SIGINT, and SIGQUIT are normally sent on terminal close,
      // ctrl-c, and ctrl-\, respectively, and are sent to all processes in the
      // foreground process group. this includes child processes, so we ignore
      // the signal and wait for them to exit
      Signal::Hangup | Signal::Interrupt | Signal::Quit => {}
      #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
      ))]
      Signal::Info => {
        let id = process::id();
        if self.children.is_empty() {
          eprintln!("just {id}: no child processes");
        } else {
          let n = self.children.len();

          let mut message = format!(
            "just {id}: {n} child {}:\n",
            if n == 1 { "process" } else { "processes" }
          );

          for (&child, command) in &self.children {
            use std::fmt::Write;
            writeln!(message, "{child}: {command:?}").unwrap();
          }

          eprint!("{message}");
        }
      }
      // SIGTERM is the default signal sent by kill. forward it to child
      // processes and wait for them to exit
      Signal::Terminate =>
      {
        #[cfg(not(windows))]
        for &child in self.children.keys() {
          if self.verbosity.loquacious() {
            eprintln!("just: sending SIGTERM to child process {child}");
          }
          nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(child),
            Some(Signal::Terminate.into()),
          )
          .ok();
        }
      }
    }
  }

  pub(crate) fn spawn<T>(
    mut command: Command,
    config: &Config,
    f: impl Fn(process::Child) -> io::Result<T>,
  ) -> (io::Result<T>, Option<Signal>) {
    let mut instance = Self::instance();

    let color = config.color.context().stderr();
    let pfx = color.prefix();
    let sfx = color.suffix();

    // Print an xtrace of run commands.
    if config.verbosity.grandiloquent() {
      // At the highest verbosity level, print the exact command as-is.
      eprintln!("{pfx}+ {command:?}{sfx}");
    } else if config.verbosity.loquacious() {
      // For the second highest verbosity level, reconstruct the command but don't include
      // environment (can be quite noisy with many `export`ed variables).
      let mut dbg_cmd = Command::new(command.get_program());
      dbg_cmd.args(command.get_args());
      if let Some(cwd) = command.get_current_dir() {
        dbg_cmd.current_dir(cwd);
      }
      eprintln!("{pfx}+ {dbg_cmd:?}{sfx}");
    }

    let child = match command.spawn() {
      Err(err) => return (Err(err), None),
      Ok(child) => child,
    };

    let pid = match child.id().try_into() {
      Err(err) => {
        return (
          Err(io::Error::other(format!("invalid child PID: {err}"))),
          None,
        )
      }
      Ok(pid) => pid,
    };

    instance.children.insert(pid, command);

    drop(instance);

    let result = f(child);

    let mut instance = Self::instance();

    instance.children.remove(&pid);

    (result, instance.caught)
  }
}
