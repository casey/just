use super::*;

/// The top-level type produced by the parser. Not all successful parses result
/// in valid justfiles, so additional consistency checks and name resolution
/// are performed by the `Analyzer`, which produces a `Justfile` from an `Ast`.
#[derive(Debug, Clone)]
pub(crate) struct Ast<'src> {
  pub(crate) items: Vec<Item<'src>>,
  pub(crate) list_features: Vec<(ListFeature, Token<'src>)>,
  pub(crate) module_path: Modulepath,
  pub(crate) unstable_features: BTreeSet<UnstableFeature>,
  pub(crate) warnings: Vec<Warning>,
  pub(crate) working_directory: PathBuf,
}

impl Ast<'_> {
  pub(crate) fn indentation(&self) -> Option<Indentation> {
    self.items.iter().find_map(|item| {
      if let Item::Setting(set) = item
        && let Setting::Indentation(Expression::StringLiteral { string_literal }) = &set.value
      {
        string_literal.cooked.parse().ok()
      } else {
        None
      }
    })
  }
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

      if let Some(i) = i.checked_sub(1)
        && let Some(last) = self.items.get(i)
        && !matches!(last, Item::Newline)
      {
        write!(f, " ")?;
      }

      write!(f, "{}", item.color_display(color))?;
    }

    writeln!(f)?;

    Ok(())
  }
}
