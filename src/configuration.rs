use common::*;

pub const DEFAULT_SHELL: &'static str = "sh";

pub struct Configuration<'a> {
  pub dry_run:   bool,
  pub evaluate:  bool,
  pub highlight: bool,
  pub overrides: Map<&'a str, &'a str>,
  pub quiet:     bool,
  pub shell:     &'a str,
  pub color:     Color,
  pub verbose:   bool,
}

impl<'a> Default for Configuration<'a> {
  fn default() -> Configuration<'static> {
    Configuration {
      dry_run:   false,
      evaluate:  false,
      highlight: false,
      overrides: empty(),
      quiet:     false,
      shell:     DEFAULT_SHELL,
      color:     default(),
      verbose:   false,
    }
  }
}
