use super::*;

pub struct Count<T: Display>(pub T, pub usize);

impl<T: Display> Display for Count<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.1 == 1 {
      write!(f, "{}", self.0)
    } else {
      write!(f, "{}s", self.0)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn count() {
    assert_eq!(Count("dog", 0).to_string(), "dogs");
    assert_eq!(Count("dog", 1).to_string(), "dog");
    assert_eq!(Count("dog", 2).to_string(), "dogs");
  }
}
