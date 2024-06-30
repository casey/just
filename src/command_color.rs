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

impl Into<ansi_term::Color> for CommandColor {
  fn into(self) -> ansi_term::Color {
    match self {
      Self::Black => ansi_term::Color::Black,
      Self::Blue => ansi_term::Color::Blue,
      Self::Cyan => ansi_term::Color::Cyan,
      Self::Green => ansi_term::Color::Green,
      Self::Purple => ansi_term::Color::Purple,
      Self::Red => ansi_term::Color::Red,
      Self::Yellow => ansi_term::Color::Yellow,
    }
  }
}
