use crate::common::*;

#[derive(Debug, Eq, PartialEq, IntoStaticStr, Display, Copy, Clone, EnumString)]
#[strum(serialize_all = "kebab_case")]
pub(crate) enum Keyword {
  Alias,
  AllowDuplicateRecipes,
  DotenvLoad,
  DotenvFilenames,
  Else,
  Export,
  False,
  If,
  PositionalArguments,
  Set,
  Shell,
  True,
  WindowsPowershell,
}

impl Keyword {
  pub(crate) fn from_lexeme(lexeme: &str) -> Option<Keyword> {
    lexeme.parse().ok()
  }

  pub(crate) fn lexeme(self) -> &'static str {
    self.into()
  }
}

impl<'a> PartialEq<&'a str> for Keyword {
  fn eq(&self, other: &&'a str) -> bool {
    self.lexeme() == *other
  }
}
