use nu_ansi_term::{AnsiGenericString, Color, Prefix};

#[derive(Default)]
pub(crate) struct Style(nu_ansi_term::Style);

impl Style {
  pub(crate) fn bold(&mut self) {
    self.0 = self.0.bold();
  }

  pub(crate) fn fg(&mut self, color: Color) {
    self.0 = self.0.fg(color);
  }

  pub(crate) fn new() -> Self {
    Self::default()
  }

  pub(crate) fn paint<'a>(&self, text: &'a str) -> AnsiGenericString<'a, str> {
    self.0.paint(text)
  }

  pub(crate) fn prefix(&self) -> Prefix {
    self.0.prefix()
  }
}
