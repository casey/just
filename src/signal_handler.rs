use super::*;

use command_group::{CommandGroup, GroupChild};
use signal_hook::consts::signal::*;
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;
use signal_hook::iterator::exfiltrator::WithOrigin;
use signal_hook::iterator::SignalsInfo;
use std::io::Read;
use std::process::Command;
use std::sync::{atomic::AtomicBool, Arc};

pub(crate) struct SignalHandler {
  verbosity: Verbosity,
  signals: Option<SignalsInfo<WithOrigin>>,
}

impl SignalHandler {
  pub(crate) fn install(verbosity: Verbosity) -> Result<(), std::io::Error> {
    let mut instance = Self::instance();
    instance.verbosity = verbosity;

    let signal_flag = Arc::new(AtomicBool::new(false));

    const MISC_SIGNALS: &[i32] = &[
      #[cfg(not(windows))]
      SIGHUP,
      // SIGABRT,
      // SIGPIPE,
      // SIGALRM,
    ];

    let mut sigs = MISC_SIGNALS.to_vec();
    sigs.extend(TERM_SIGNALS);

    let signals = SignalsInfo::<WithOrigin>::new(&sigs)?;
    instance.signals = Some(signals);

    for sig in TERM_SIGNALS {
      // NOTE This could be set before the register handler, in order to terminate in a double ctrl-c
      // flag::register_conditional_shutdown(*sig, 1, Arc::clone(&signal_flag))?;
      flag::register(*sig, Arc::clone(&signal_flag))?;
    }

    for sig in MISC_SIGNALS {
      flag::register(*sig, Arc::clone(&signal_flag))?;
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
            message: format!("interrupt handler mutex poisoned: {poison_error}"),
          }
          .color_display(Color::auto().stderr())
        );
        process::exit(EXIT_FAILURE);
      }
    }
  }

  const fn new() -> Self {
    Self {
      verbosity: Verbosity::default(),
      signals: None,
    }
  }

  fn wait_child(mut child: GroupChild) -> std::io::Result<(GroupChild, ExitStatus)> {
    let mut instance = Self::instance();
    loop {
      if let Some(signals) = instance.signals.as_mut() {
        for signal_info in signals.pending() {
          unsafe {
            // Send signal to the child process group, thus the negative pid
            libc::kill(-(child.id() as i32), signal_info.signal);
          }
        }
      }
      match child.try_wait() {
        Ok(None) => {
          std::thread::sleep(std::time::Duration::from_millis(10));
          // If the child process is still running, continue polling
          continue;
        }
        Ok(Some(status)) => {
          // If the child process has exited, break the loop
          return Ok((child, status));
        }
        Err(e) => {
          return Err(e);
        }
      }
    }
  }

  pub(crate) fn guard_output(mut command: Command) -> Result<String, OutputError> {
    let child = command
      .stdout(std::process::Stdio::piped())
      .group_spawn()
      .unwrap();
    match Self::wait_child(child) {
      Ok((mut child, status)) => {
        if let Some(code) = status.code() {
          if code != 0 {
            return Err(OutputError::Code(code));
          }
        } else {
          let signal = Platform::signal_from_exit_status(status);
          return Err(match signal {
            Some(signal) => OutputError::Signal(signal),
            None => OutputError::Unknown,
          });
        }
        match child.inner().stdout.as_mut() {
          Some(stdout) => {
            let mut buffer = Vec::new();
            stdout.read_to_end(&mut buffer).unwrap();
            match str::from_utf8(buffer.as_slice()) {
              Err(error) => Err(OutputError::Utf8(error)),
              Ok(utf8) => Ok(
                if utf8.ends_with('\n') {
                  &utf8[0..utf8.len() - 1]
                } else if utf8.ends_with("\r\n") {
                  &utf8[0..utf8.len() - 2]
                } else {
                  utf8
                }
                .to_owned(),
              ),
            }
          }
          None => Err(OutputError::Unknown),
        }
      }
      Err(error) => Err(OutputError::Io(error)),
    }
  }

  pub(crate) fn guard(mut command: Command) -> Result<std::process::ExitStatus, std::io::Error> {
    let child = command.group_spawn().unwrap();
    match Self::wait_child(child) {
      Ok((_, status)) => Ok(status),
      Err(error) => Err(error),
    }
  }
}
