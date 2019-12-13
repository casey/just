use crate::common::*;

pub(crate) trait ErrorResultExt<T> {
  fn eprint(self, color: Color) -> Result<T, i32>;
}

impl<T, E: Error> ErrorResultExt<T> for Result<T, E> {
  fn eprint(self, color: Color) -> Result<T, i32> {
    match self {
      Ok(ok) => Ok(ok),
      Err(error) => {
        if color.stderr().active() {
          eprintln!("{}: {:#}", color.stderr().error().paint("error"), error);
        } else {
          eprintln!("error: {}", error);
        }

        Err(error.code())
      }
    }
  }
}
