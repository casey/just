use super::*;

/// The top-level type produced by the parser. Not all successful parses result
/// in valid justfiles, so additional consistency checks and name resolution
/// are performed by the `Analyzer`, which produces a `Justfile` from an `Ast`.
#[derive(Debug, Clone)]
pub(crate) struct Ast<'src> {
  pub(crate) items: Vec<Item<'src>>,
  pub(crate) warnings: Vec<Warning>,
  pub(crate) working_directory: PathBuf,
}

impl<'src> Display for Ast<'src> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut iter = self.items.iter().peekable();

    while let Some(item) = iter.next() {
      writeln!(f, "{item}")?;

      if let Some(next_item) = iter.peek() {
        if matches!(item, Item::Recipe(_))
          || mem::discriminant(item) != mem::discriminant(next_item)
        {
          writeln!(f)?;
        }
      }
    }

    Ok(())
  }
}
