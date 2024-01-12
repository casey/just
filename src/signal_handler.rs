use {
  super::*,
  command_group::{CommandGroup, GroupChild},
  signal_hook::{
    consts::{signal::SIGHUP, TERM_SIGNALS},
    iterator::SignalsInfo,
  },
  std::{io::Read, process::Command},
};

pub(crate) struct SignalHandler {
  signals_info: Option<SignalsInfo>,
}

impl SignalHandler {
  pub(crate) fn install() -> Result<(), std::io::Error> {
    *Self::instance() = Self {
      signals_info: Some(SignalsInfo::new(
        iter::once(SIGHUP)
          .chain(TERM_SIGNALS.iter().copied())
          .collect::<Vec<i32>>(),
      )?),
    };

    Ok(())
  }

  fn instance() -> MutexGuard<'static, Self> {
    static INSTANCE: Mutex<SignalHandler> = Mutex::new(SignalHandler { signals_info: None });

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
    let mut instance = Self::instance();
    loop {
      if let Some(signals_info) = instance.signals_info.as_mut() {
        for signal in signals_info.pending() {
          let child_pid = child.id();
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
