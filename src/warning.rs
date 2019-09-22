use crate::common::*;
use crate::misc::write_message_context;

use Warning::*;

#[derive(Debug)]
pub(crate) enum Warning<'a> {
  DeprecatedEquals { equals: Token<'a> },
}

impl Warning<'_> {
  fn context(&self) -> Option<&Token> {
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
      write_message_context(
        f,
        Color::fmt(f).warning(),
        token.text,
        token.offset,
        token.line,
        token.column,
        token.lexeme().len(),
      )?;
    }

    Ok(())
  }
}
