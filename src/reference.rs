use super::*;

pub(crate) enum Reference<'src> {
  Call { name: Name<'src>, arguments: usize },
  Variable(Name<'src>),
}
