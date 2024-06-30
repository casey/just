use super::*;

#[derive(Copy, Clone, ValueEnum)]
pub(crate) enum CommandColor {
  Black,
  Blue,
  Cyan,
  Green,
  Purple,
  Red,
  Yellow,
}

impl From<CommandColor> for ansi_term::Color {
  fn from(command_color: CommandColor) -> Self {
    match command_color {
      CommandColor::Black => Self::Black,
      CommandColor::Blue => Self::Blue,
      CommandColor::Cyan => Self::Cyan,
      CommandColor::Green => Self::Green,
      CommandColor::Purple => Self::Purple,
      CommandColor::Red => Self::Red,
      CommandColor::Yellow => Self::Yellow,
    }
  }
}
