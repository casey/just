use crate::common::*;

pub(crate) const DEFAULT_SHELL: &str = "sh";

pub(crate) struct Configuration<'a> {
  pub(crate) dry_run: bool,
  pub(crate) evaluate: bool,
  pub(crate) highlight: bool,
  pub(crate) overrides: BTreeMap<&'a str, &'a str>,
  pub(crate) quiet: bool,
  pub(crate) shell: &'a str,
  pub(crate) color: Color,
  pub(crate) verbosity: Verbosity,
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
