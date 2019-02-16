use common::*;

pub const DEFAULT_SHELL: Shell = Shell::ShLike("sh");

pub struct Configuration<'a> {
  pub dry_run: bool,
  pub evaluate: bool,
  pub highlight: bool,
  pub overrides: Map<&'a str, &'a str>,
  pub quiet: bool,
  pub shell: Shell<'a>,
  pub color: Color,
  pub verbosity: Verbosity,
}

impl<'a> Default for Configuration<'a> {
  fn default() -> Configuration<'static> {
    Configuration {
      dry_run: false,
      evaluate: false,
      highlight: false,
      overrides: empty(),
      quiet: false,
      shell: DEFAULT_SHELL,
      color: default(),
      verbosity: Verbosity::from_flag_occurrences(0),
    }
  }
}
