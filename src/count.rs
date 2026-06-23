use super::*;

pub(crate) struct Count<T: Display> {
  count: usize,
  irregular: Option<T>,
  noun: T,
  numbered: bool,
}

impl<T: Display> Count<T> {
  pub(crate) fn numbered(noun: T, count: impl Borrow<usize>) -> Self {
    Self {
      count: *count.borrow(),
      irregular: None,
      noun,
      numbered: true,
    }
  }

  pub(crate) fn numbered_irregular(noun: T, irregular: T, count: impl Borrow<usize>) -> Self {
    Self {
      count: *count.borrow(),
      irregular: Some(irregular),
      noun,
      numbered: true,
    }
  }

  pub(crate) fn unnumbered(noun: T, count: impl Borrow<usize>) -> Self {
    Self {
      count: *count.borrow(),
      irregular: None,
      noun,
      numbered: false,
    }
  }
}

impl<T: Display> Display for Count<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.numbered {
      write!(f, "{} ", self.count)?;
    }

    if let Some(irregular) = &self.irregular {
      if self.count == 1 {
        write!(f, "{}", self.noun)?;
      } else {
        write!(f, "{irregular}")?;
      }
    } else {
      write!(f, "{}", self.noun)?;
      if self.count != 1 {
        write!(f, "s")?;
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn count() {
    assert_eq!(Count::numbered("dog", 0).to_string(), "0 dogs");
    assert_eq!(Count::numbered("dog", 1).to_string(), "1 dog");
    assert_eq!(Count::numbered("dog", 2).to_string(), "2 dogs");
    assert_eq!(
      Count::numbered_irregular("foot", "feet", 0).to_string(),
      "0 feet"
    );
    assert_eq!(
      Count::numbered_irregular("foot", "feet", 1).to_string(),
      "1 foot"
    );
    assert_eq!(
      Count::numbered_irregular("foot", "feet", 2).to_string(),
      "2 feet"
    );
    assert_eq!(Count::unnumbered("dog", 0).to_string(), "dogs");
    assert_eq!(Count::unnumbered("dog", 1).to_string(), "dog");
    assert_eq!(Count::unnumbered("dog", 2).to_string(), "dogs");
  }
}
