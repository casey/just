use ansi_term::Colour;
use {
  super::*,
  ansi_term::{ANSIGenericString, Color::*, Prefix, Style, Suffix},
  atty::Stream,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Color {
  use_color: UseColor,
  atty: bool,
  style: Style,
}

impl Color {
  fn restyle(self, style: Style) -> Self {
    Self { style, ..self }
  }

  fn redirect(self, stream: Stream) -> Self {
    Self {
      atty: atty::is(stream),
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

  pub(crate) fn auto() -> Self {
    Self {
      use_color: UseColor::Auto,
      ..Color::default()
    }
  }

  pub(crate) fn always() -> Self {
    Self {
      use_color: UseColor::Always,
      ..Color::default()
    }
  }

  pub(crate) fn never() -> Self {
    Self {
      use_color: UseColor::Never,
      ..Color::default()
    }
  }

  pub(crate) fn stderr(self) -> Self {
    self.redirect(Stream::Stderr)
  }

  pub(crate) fn stdout(self) -> Self {
    self.redirect(Stream::Stdout)
  }

  pub(crate) fn doc(self) -> Self {
    self.restyle(Style::new().fg(Blue))
  }

  pub(crate) fn error(self) -> Self {
    self.restyle(Style::new().fg(Red).bold())
  }

  pub(crate) fn warning(self) -> Self {
    self.restyle(Style::new().fg(Yellow).bold())
  }

  pub(crate) fn banner(self) -> Self {
    self.restyle(Style::new().fg(Cyan).bold())
  }

  pub(crate) fn command(self, foreground_color: Option<Colour>) -> Self {
    let new_style = Style {
      foreground: foreground_color,
      is_bold: true,
      ..Style::default()
    };
    self.restyle(new_style)
  }

  pub(crate) fn parameter(self) -> Self {
    self.restyle(Style::new().fg(Cyan))
  }

  pub(crate) fn message(self) -> Self {
    self.restyle(Style::new().bold())
  }

  pub(crate) fn annotation(self) -> Self {
    self.restyle(Style::new().fg(Purple))
  }

  pub(crate) fn string(self) -> Self {
    self.restyle(Style::new().fg(Green))
  }

  pub(crate) fn diff_added(self) -> Self {
    self.restyle(Style::new().fg(Green))
  }

  pub(crate) fn diff_deleted(self) -> Self {
    self.restyle(Style::new().fg(Red))
  }

  pub(crate) fn active(&self) -> bool {
    match self.use_color {
      UseColor::Always => true,
      UseColor::Never => false,
      UseColor::Auto => self.atty,
    }
  }

  pub(crate) fn paint<'a>(&self, text: &'a str) -> ANSIGenericString<'a, str> {
    self.effective_style().paint(text)
  }

  pub(crate) fn prefix(&self) -> Prefix {
    self.effective_style().prefix()
  }

  pub(crate) fn suffix(&self) -> Suffix {
    self.effective_style().suffix()
  }
}

impl Default for Color {
  fn default() -> Self {
    Self {
      use_color: UseColor::Auto,
      atty: false,
      style: Style::new(),
    }
  }
}
