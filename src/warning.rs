use crate::common::*;

use Warning::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Warning<'src> {
  DeprecatedEquals { equals: Token<'src> },
}

impl<'src> Warning<'src> {
  fn context(&self) -> Option<&Token<'src>> {
    match self {
      DeprecatedEquals { equals } => Some(equals),
    }
  }
}

impl Display for Warning<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let warning = Color::fmt(f).warning();
    let message = Color::fmt(f).message();

    write!(f, "{} {}", warning.paint("warning:"), message.prefix())?;

    match self {
      DeprecatedEquals { .. } => {
        writeln!(
          f,
          "`=` in assignments, exports, and aliases is being phased out on favor of `:=`"
        )?;
        write!(
          f,
          "Please see this issue for more details: https://github.com/casey/just/issues/379"
        )?;
      }
    }

    write!(f, "{}", message.suffix())?;

    if let Some(token) = self.context() {
      writeln!(f)?;
      token.write_context(f, Color::fmt(f).warning())?;
    }

    Ok(())
  }
}
