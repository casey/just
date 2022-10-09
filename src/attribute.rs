use super::*;

#[derive(EnumString)]
#[strum(serialize_all = "kebab_case")]
pub(crate) enum Attribute {
  NoExitMessage,
}

impl Attribute {
  pub(crate) fn from_name(name: Name) -> Option<Attribute> {
    name.lexeme().parse().ok()
  }
}
