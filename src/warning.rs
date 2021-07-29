use crate::common::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Warning {
  DotenvLoad,
}

impl Warning {
  fn context(&self) -> Option<&Token> {
    match self {
      Self::DotenvLoad => None,
    }
  }
}

impl ColorDisplay for Warning {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    let warning = color.warning();
    let message = color.message();

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

This warning may also be silenced by setting the `JUST_SUPPRESS_DOTENV_LOAD_WARNING`
environment variable to `1`. This can be used to silence the warning globally by
adding the following line to your shell rc file:

  export JUST_SUPPRESS_DOTENV_LOAD_WARNING=1

See https://github.com/casey/just/issues/469 for more details.")?;
      },
    }

    write!(f, "{}", message.suffix())?;

    if let Some(token) = self.context() {
      writeln!(f)?;
      write!(f, "{}", token.color_display(color))?;
    }

    Ok(())
  }
}
