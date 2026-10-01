use super::*;

macro_rules! print {
  ($($arg:tt)*) => {
    $crate::print::print(format_args!($($arg)*))?
  };
}

macro_rules! println {
  () => {
    print!("\n")
  };
  ($($arg:tt)*) => {
    print!("{}\n", format_args!($($arg)*))
  };
}

pub(crate) fn print(args: fmt::Arguments) -> RunResult<'static> {
  match io::stdout().write_fmt(args) {
    Err(io_error) if io_error.kind() != io::ErrorKind::BrokenPipe => {
      Err(Error::StdoutIo { io_error })
    }
    _ => Ok(()),
  }
}
