use common::*;

use color::Color;

#[derive(Default)]
pub struct Configuration<'a> {
  pub dry_run:   bool,
  pub evaluate:  bool,
  pub highlight: bool,
  pub overrides: Map<&'a str, &'a str>,
  pub quiet:     bool,
  pub shell:     Option<&'a str>,
  pub color:     Color,
  pub verbose:   bool,
}
