use {
  super::*,
  command_group::{CommandGroup, GroupChild},
  crossbeam_queue::SegQueue,
  libc::c_int,
  signal_hook::consts::TERM_SIGNALS,
  std::{io::Read, process::Command},
};

pub(crate) struct SignalHandler {
  signals: SegQueue<c_int>,
}

impl SignalHandler {
  pub(crate) fn install() -> Result<(), std::io::Error> {
    #[cfg(unix)]
    let signals = iter::once(signal_hook::consts::signal::SIGHUP)
      .chain(TERM_SIGNALS.iter().copied())
      .collect::<Vec<c_int>>();

    #[cfg(windows)]
    let signals = TERM_SIGNALS;

    signals.iter().copied().for_each(|signal| {
      let res = unsafe {
        signal_hook::low_level::register(signal, move || {
          println!("Enqueuing signal {signal:?}");
          Self::instance().signals.push(signal);
        })
      };
      if let Err(error) = res {
        eprintln!(
          "{}",
          Error::Internal {
            message: format!("Could not register signal handler: {error}"),
          }
          .color_display(Color::auto().stderr())
        );
        process::exit(EXIT_FAILURE);
      }
    });

    Ok(())
  }

  fn instance() -> MutexGuard<'static, Self> {
    static INSTANCE: Mutex<SignalHandler> = Mutex::new(SignalHandler {
      signals: SegQueue::<c_int>::new(),
    });

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

  fn wait_child(mut child: GroupChild) -> std::io::Result<(GroupChild, ExitStatus)> {
    loop {
      if let Some(signal) = Self::instance().signals.pop() {
        println!("Popping signal {signal:?}");
        let child_pid = child.id();
        #[cfg(unix)]
        if let Ok(pid) = i32::try_from(child_pid) {
          unsafe {
            // Forward signal to the process group using negative pid
            libc::kill(-pid, signal);
          }
        } else {
          eprintln!(
            "{}",
            Error::Internal {
              message: format!("Could not convert child pid to i32: {child_pid}"),
            }
            .color_display(Color::auto().stderr())
          );
          process::exit(EXIT_FAILURE);
        }

        #[cfg(windows)]
        unsafe {
          windows::Win32::System::Console::GenerateConsoleCtrlEvent(signal as u32, child_pid);
        }
      }
      match child.try_wait() {
        Ok(None) => {
          std::thread::sleep(std::time::Duration::from_millis(50));
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
      .map_err(OutputError::Io)?;
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
            stdout.read_to_end(&mut buffer).map_err(OutputError::Io)?;
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
    let child = command.group_spawn()?;
    match Self::wait_child(child) {
      Ok((_, status)) => Ok(status),
      Err(error) => Err(error),
    }
  }
}
