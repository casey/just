use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Warning {
  // Remove this on 2021-07-01.
  #[allow(dead_code)]
  DotenvLoad,
}

impl Warning {
  fn context(&self) -> Option<&Token> {
    match self {
      Self::DotenvLoad => None,
    }
  }
}

impl Display for Warning {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let warning = Color::fmt(f).warning();
    let message = Color::fmt(f).message();

    write!(f, "{} {}", warning.paint("warning:"), message.prefix())?;

    match self {
      Self::DotenvLoad => {
        #[rustfmt::skip]
        write!(f, "\
A `.env` file was found and loaded, but this behavior will change in the future.
To silence this warning and continue loading `.env` files, add:

    set dotenv-load := true

To silence this warning and stop loading `.env` files, add:

    set dotenv-load := false

See https://github.com/casey/just/issues/469 for more details.")?;
      },
    }

    write!(f, "{}", message.suffix())?;

    if let Some(token) = self.context() {
      writeln!(f)?;
      token.write_context(f, Color::fmt(f).warning())?;
    }

    Ok(())
  }
}
