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
    let mut newlines = 0;
    for (i, item) in self.items.iter().enumerate() {
      if matches!(item, Item::Newline) {
        newlines += 1;
        continue;
      }

      match newlines {
        0 => {}
        1 => writeln!(f)?,
        _ => {
          writeln!(f)?;
          writeln!(f)?;
        }
      }
      newlines = 0;

      if let Some(i) = i.checked_sub(1) {
        if let Some(last) = self.items.get(i) {
          if !matches!(last, Item::Newline) {
            write!(f, " ")?;
          }
        }
      }

      write!(f, "{}", item.color_display(color))?;
    }

    writeln!(f)?;

    Ok(())
  }
}
