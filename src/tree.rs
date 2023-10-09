use {
  super::*,
  std::{borrow::Cow, mem},
};

/// Construct a `Tree` from a symbolic expression literal. This macro, and the
/// Tree type, are only used in the Parser unit tests, providing a concise
/// notation for representing the expected results of parsing a given string.
macro_rules! tree {
  {
    ($($child:tt)*)
  } => {
    $crate::tree::Tree::List(vec![$(tree!($child),)*])
  };

  {
    $atom:ident
  } => {
    $crate::tree::Tree::atom(stringify!($atom))
  };

  {
    $atom:literal
  } => {
    $crate::tree::Tree::atom(format!("\"{}\"", $atom))
  };

  {
    #
  } => {
    $crate::tree::Tree::atom("#")
  };

  {
    +
  } => {
    $crate::tree::Tree::atom("+")
  };

  {
    *
  } => {
    $crate::tree::Tree::atom("*")
  };

  {
    &&
  } => {
    $crate::tree::Tree::atom("&&")
  };

  {
    ==
  } => {
    $crate::tree::Tree::atom("==")
  };

  {
    !=
  } => {
    $crate::tree::Tree::atom("!=")
  };
}

/// A `Tree` is either…
#[derive(Debug, PartialEq)]
pub(crate) enum Tree<'text> {
  /// …an atom containing text, or…
  Atom(Cow<'text, str>),
  /// …a list containing zero or more `Tree`s.
  List(Vec<Tree<'text>>),
}

impl<'text> Tree<'text> {
  /// Construct an Atom from a text scalar
  pub(crate) fn atom(text: impl Into<Cow<'text, str>>) -> Tree<'text> {
    Tree::Atom(text.into())
  }

  /// Construct a List from an iterable of trees
  pub(crate) fn list(children: impl IntoIterator<Item = Tree<'text>>) -> Tree<'text> {
    Tree::List(children.into_iter().collect())
  }

  /// Convenience function to create an atom containing quoted text
  pub(crate) fn string(contents: impl AsRef<str>) -> Tree<'text> {
    Tree::atom(format!("\"{}\"", contents.as_ref()))
  }

  /// Push a child node into self, turning it into a List if it was an Atom
  pub(crate) fn push(self, tree: impl Into<Tree<'text>>) -> Tree<'text> {
    match self {
      Tree::List(mut children) => {
        children.push(tree.into());
        Tree::List(children)
      }
      Tree::Atom(text) => Tree::List(vec![Tree::Atom(text), tree.into()]),
    }
  }

  /// Extend a self with a tail of Trees, turning self into a List if it was an
  /// Atom
  pub(crate) fn extend<I, T>(self, tail: I) -> Tree<'text>
  where
    I: IntoIterator<Item = T>,
    T: Into<Tree<'text>>,
  {
    // Tree::List(children.into_iter().collect())
    let mut head = match self {
      Tree::List(children) => children,
      Tree::Atom(text) => vec![Tree::Atom(text)],
    };

    for child in tail {
      head.push(child.into());
    }

    Tree::List(head)
  }

  /// Like `push`, but modify self in-place
  pub(crate) fn push_mut(&mut self, tree: impl Into<Tree<'text>>) {
    *self = mem::replace(self, Tree::List(Vec::new())).push(tree.into());
  }
}

impl Display for Tree<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Tree::List(children) => {
        write!(f, "(")?;

        for (i, child) in children.iter().enumerate() {
          if i > 0 {
            write!(f, " ")?;
          }
          write!(f, "{child}")?;
        }

        write!(f, ")")
      }
      Tree::Atom(text) => write!(f, "{text}"),
    }
  }
}

impl<'text, T> From<T> for Tree<'text>
where
  T: Into<Cow<'text, str>>,
{
  fn from(text: T) -> Tree<'text> {
    Tree::Atom(text.into())
  }
}
