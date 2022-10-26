use super::*;

#[derive(EnumString, PartialEq, Debug, Clone, Serialize)]
#[strum(serialize_all = "kebab_case")]
pub(crate) enum Attribute {
  Linux,
  Macos,
  NoExitMessage,
  Unix,
  Windows,
}

impl Attribute {
  pub(crate) fn from_name(name: Name) -> Option<Attribute> {
    name.lexeme().parse().ok()
  }
}
