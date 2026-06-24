use nu_ansi_term::{AnsiGenericString, Color, Prefix};

#[derive(Default)]
pub(crate) struct Style(nu_ansi_term::Style);

impl Style {
  pub(crate) fn bg(&mut self, color: Color) {
    self.0 = self.0.on(color);
  }

  pub(crate) fn blink(&mut self) {
    self.0 = self.0.blink();
  }

  pub(crate) fn bold(&mut self) {
    self.0 = self.0.bold();
  }

  pub(crate) fn dimmed(&mut self) {
    self.0 = self.0.dimmed();
  }

  pub(crate) fn fg(&mut self, color: Color) {
    self.0 = self.0.fg(color);
  }

  pub(crate) fn hidden(&mut self) {
    self.0 = self.0.underline();
  }

  pub(crate) fn italic(&mut self) {
    self.0 = self.0.italic();
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

  pub(crate) fn reverse(&mut self) {
    self.0 = self.0.reverse();
  }

  pub(crate) fn strikethrough(&mut self) {
    self.0 = self.0.strikethrough();
  }

  pub(crate) fn underline(&mut self) {
    self.0 = self.0.underline();
  }
}
