use super::*;

pub(crate) struct Count {
  count: usize,
  irregular: Option<&'static str>,
  noun: &'static str,
  numbered: bool,
}

impl Count {
  pub(crate) fn numbered(noun: &'static str, count: impl Borrow<usize>) -> Self {
    Self {
      count: *count.borrow(),
      irregular: None,
      noun,
      numbered: true,
    }
  }

  pub(crate) fn numbered_irregular(
    noun: &'static str,
    irregular: &'static str,
    count: impl Borrow<usize>,
  ) -> Self {
    Self {
      count: *count.borrow(),
      irregular: Some(irregular),
      noun,
      numbered: true,
    }
  }

  pub(crate) fn unnumbered(noun: &'static str, count: impl Borrow<usize>) -> Self {
    Self {
      count: *count.borrow(),
      irregular: None,
      noun,
      numbered: false,
    }
  }
}

impl Display for Count {
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
    #[track_caller]
    fn case(count: Count, expected: &str) {
      assert_eq!(count.to_string(), expected);
    }

    case(Count::numbered("dog", 0), "0 dogs");
    case(Count::numbered("dog", 1), "1 dog");
    case(Count::numbered("dog", 2), "2 dogs");
    case(Count::numbered_irregular("foot", "feet", 0), "0 feet");
    case(Count::numbered_irregular("foot", "feet", 1), "1 foot");
    case(Count::numbered_irregular("foot", "feet", 2), "2 feet");
    case(Count::unnumbered("dog", 0), "dogs");
    case(Count::unnumbered("dog", 1), "dog");
    case(Count::unnumbered("dog", 2), "dogs");
  }
}
