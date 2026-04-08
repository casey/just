use super::*;

/// The top-level type produced by the parser. Not all successful parses result
/// in valid justfiles, so additional consistency checks and name resolution
/// are performed by the `Analyzer`, which produces a `Justfile` from an `Ast`.
#[derive(Debug, Clone)]
pub(crate) struct Ast<'src> {
  pub(crate) items: Vec<Item<'src>>,
  pub(crate) modulepath: Modulepath,
  pub(crate) unstable_features: BTreeSet<UnstableFeature>,
  pub(crate) warnings: Vec<Warning>,
  pub(crate) working_directory: PathBuf,
}

impl ColorDisplay for Ast<'_> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    let mut newline = false;
    for item in &self.items {
      if matches!(item, Item::Newline) {
        newline = true;
        continue;
      }

      if newline {
        writeln!(f)?;
        newline = false;
      }

      writeln!(f, "{}", item.color_display(color))?;
    }

    Ok(())
  }
}
