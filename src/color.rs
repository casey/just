extern crate ansi_term;
extern crate atty;

use prelude::*;
use self::ansi_term::{Style, Prefix, Suffix, ANSIGenericString};
use self::ansi_term::Color::*;
use self::atty::is as is_atty;
use self::atty::Stream;

#[derive(Copy, Clone)]
pub enum UseColor {
  Auto,
  Always,
  Never,
}

#[derive(Copy, Clone)]
pub struct Color {
  use_color: UseColor,
  atty:      bool,
  style:     Style,
}

impl Default for Color {
  fn default() -> Color {
    Color {
      use_color: UseColor::Never,
      atty:      false,
      style:     Style::new(),
    }
  }
}

impl Color {
  fn restyle(self, style: Style) -> Color {
    Color {
      style: style,
      ..self
    }
  }

  fn redirect(self, stream: Stream) -> Color {
    Color {
      atty: is_atty(stream),
      ..self
    }
  }

  fn effective_style(&self) -> Style {
    if self.active() {
      self.style
    } else {
      Style::new()
    }
  }

  pub fn fmt(fmt: &fmt::Formatter) -> Color {
    if fmt.alternate() {
      Color::always()
    } else {
      Color::never()
    }
  }

  pub fn auto() -> Color {
    Color {
      use_color: UseColor::Auto,
      ..default()
    }
  }

  pub fn always() -> Color {
    Color {
      use_color: UseColor::Always,
      ..default()
    }
  }

  pub fn never() -> Color {
    Color {
      use_color: UseColor::Never,
      ..default()
    }
  }

  pub fn stderr(self) -> Color {
    self.redirect(Stream::Stderr)
  }

  pub fn stdout(self) -> Color {
    self.redirect(Stream::Stdout)
  }

  pub fn doc(self) -> Color {
    self.restyle(Style::new().fg(Blue))
  }

  pub fn error(self) -> Color {
    self.restyle(Style::new().fg(Red).bold())
  }

  pub fn banner(self) -> Color {
    self.restyle(Style::new().fg(Cyan).bold())
  }

  pub fn command(self) -> Color {
    self.restyle(Style::new().bold())
  }

  pub fn parameter(self) -> Color {
    self.restyle(Style::new().fg(Cyan))
  }

  pub fn message(self) -> Color {
    self.restyle(Style::new().bold())
  }

  pub fn annotation(self) -> Color {
    self.restyle(Style::new().fg(Purple))
  }

  pub fn string(self) -> Color {
    self.restyle(Style::new().fg(Green))
  }

  pub fn active(&self) -> bool {
    match self.use_color {
      UseColor::Always => true,
      UseColor::Never  => false,
      UseColor::Auto   => self.atty,
    }
  }

  pub fn paint<'a>(&self, text: &'a str) -> ANSIGenericString<'a, str> {
    self.effective_style().paint(text)
  }

  pub fn prefix(&self) -> Prefix {
    self.effective_style().prefix()
  }

  pub fn suffix(&self) -> Suffix {
    self.effective_style().suffix()
  }
}
