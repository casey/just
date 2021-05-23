use crate::common::*;

/// A module, the top-level type produced by the parser. So-named because
/// although at present, all justfiles consist of a single module, in the future
/// we will likely have multi-module and multi-file justfiles.
///
/// Not all successful parses result in valid justfiles, so additional
/// consistency checks and name resolution are performed by the `Analyzer`,
/// which produces a `Justfile` from a `Module`.
#[derive(Debug)]
pub(crate) struct Module<'src> {
  /// Items in the justfile
  pub(crate) items:    Vec<Item<'src>>,
  /// Non-fatal warnings encountered during parsing
  pub(crate) warnings: Vec<Warning>,
}

impl<'src> Display for Module<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    let mut iter = self.items.iter().peekable();

    while let Some(item) = iter.next() {
      writeln!(f, "{}", item)?;

      if let Some(next_item) = iter.peek() {
        if matches!(item, Item::Recipe(_))
          || std::mem::discriminant(item) != std::mem::discriminant(next_item)
        {
          writeln!(f, "")?;
        }
      }
    }

    Ok(())
  }
}
