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
  if let Err(io_error) = io::stdout().write_fmt(args)
    && io_error.kind() != io::ErrorKind::BrokenPipe
  {
    Err(Error::StdoutIo { io_error })
  } else {
    Ok(())
  }
}
