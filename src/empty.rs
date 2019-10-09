use crate::common::*;

pub(crate) fn empty<T, C: iter::FromIterator<T>>() -> C {
  iter::empty().collect()
}
