// `Self` cannot be used where type takes generic arguments
#![allow(clippy::use_self)]

use crate::common::*;

pub struct Enclosure<T: Display> {
  enclosure: &'static str,
  value: T,
}

impl<T: Display> Enclosure<T> {
  pub fn tick(value: T) -> Enclosure<T> {
    Self {
      enclosure: "`",
      value,
    }
  }
}

impl<T: Display> Display for Enclosure<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}{}{}", self.enclosure, self.value, self.enclosure)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tick() {
    assert_eq!(Enclosure::tick("foo").to_string(), "`foo`")
  }
}
