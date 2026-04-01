use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub(crate) enum Signal {
  Hangup = 1,
  #[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
  ))]
  Info = 29,
  Interrupt = 2,
  Quit = 3,
  Terminate = 15,
}

impl Signal {
  #[cfg(not(windows))]
  pub(crate) const ALL: &'static [Self] = &[
    Self::Hangup,
    #[cfg(any(
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "ios",
      target_os = "macos",
      target_os = "netbsd",
      target_os = "openbsd",
    ))]
    Self::Info,
    Self::Interrupt,
    Self::Quit,
    Self::Terminate,
  ];

  pub(crate) fn code(self) -> i32 {
    128i32.checked_add(self.number()).unwrap()
  }

  pub(crate) fn is_fatal(self) -> bool {
    match self {
      Self::Hangup | Self::Interrupt | Self::Quit | Self::Terminate => true,
      #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
      ))]
      Self::Info => false,
    }
  }

  pub(crate) fn number(self) -> i32 {
    self as libc::c_int
  }
}

impl Display for Signal {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Hangup => "SIGHUP",
        #[cfg(any(
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "ios",
          target_os = "macos",
          target_os = "netbsd",
          target_os = "openbsd",
        ))]
        Self::Info => "SIGINFO",
        Self::Interrupt => "SIGINT",
        Self::Quit => "SIGQUIT",
        Self::Terminate => "SIGTERM",
      }
    )
  }
}

#[cfg(not(windows))]
impl From<Signal> for nix::sys::signal::Signal {
  fn from(signal: Signal) -> Self {
    match signal {
      Signal::Hangup => Self::SIGHUP,
      #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
      ))]
      Signal::Info => Self::SIGINFO,
      Signal::Interrupt => Self::SIGINT,
      Signal::Quit => Self::SIGQUIT,
      Signal::Terminate => Self::SIGTERM,
    }
  }
}

impl TryFrom<u8> for Signal {
  type Error = io::Error;

  fn try_from(n: u8) -> Result<Self, Self::Error> {
    match n {
      1 => Ok(Self::Hangup),
      #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
      ))]
      29 => Ok(Self::Info),
      2 => Ok(Self::Interrupt),
      3 => Ok(Self::Quit),
      15 => Ok(Self::Terminate),
      _ => Err(io::Error::other(format!("unexpected signal: {n}"))),
    }
  }
}

#[cfg(test)]
#[cfg(not(windows))]
mod tests {
  use super::*;

  #[test]
  fn signals_fit_in_u8() {
    for signal in Signal::ALL {
      assert!(signal.number() <= i32::from(u8::MAX));
    }
  }

  #[test]
  fn signals_have_valid_exit_codes() {
    for signal in Signal::ALL {
      signal.code();
    }
  }

  #[test]
  fn signal_numbers_are_correct() {
    for &signal in Signal::ALL {
      let n = match signal {
        Signal::Hangup => libc::SIGHUP,
        #[cfg(any(
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "ios",
          target_os = "macos",
          target_os = "netbsd",
          target_os = "openbsd",
        ))]
        Signal::Info => libc::SIGINFO,
        Signal::Interrupt => libc::SIGINT,
        Signal::Quit => libc::SIGQUIT,
        Signal::Terminate => libc::SIGTERM,
      };

      assert_eq!(signal as i32, n);

      assert_eq!(Signal::try_from(u8::try_from(n).unwrap()).unwrap(), signal);
    }
  }
}
