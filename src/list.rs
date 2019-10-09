use crate::common::*;

pub struct List<T: Display, I: Iterator<Item = T> + Clone> {
  conjunction: &'static str,
  values: I,
}

impl<T: Display, I: Iterator<Item = T> + Clone> List<T, I> {
  pub fn or<II: IntoIterator<Item = T, IntoIter = I>>(values: II) -> List<T, I> {
    List {
      conjunction: "or",
      values: values.into_iter(),
    }
  }

  pub fn and<II: IntoIterator<Item = T, IntoIter = I>>(values: II) -> List<T, I> {
    List {
      conjunction: "and",
      values: values.into_iter(),
    }
  }

  pub fn or_ticked<II: IntoIterator<Item = T, IntoIter = I>>(
    values: II,
  ) -> List<Enclosure<T>, impl Iterator<Item = Enclosure<T>> + Clone> {
    List {
      conjunction: "or",
      values: values.into_iter().map(Enclosure::tick),
    }
  }

  pub fn and_ticked<II: IntoIterator<Item = T, IntoIter = I>>(
    values: II,
  ) -> List<Enclosure<T>, impl Iterator<Item = Enclosure<T>> + Clone> {
    List {
      conjunction: "and",
      values: values.into_iter().map(Enclosure::tick),
    }
  }
}

impl<T: Display, I: Iterator<Item = T> + Clone> Display for List<T, I> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut values = self.values.clone().fuse();

    if let Some(first) = values.next() {
      write!(f, "{}", first)?;
    } else {
      return Ok(());
    }

    let second = values.next();

    if second.is_none() {
      return Ok(());
    }

    let third = values.next();

    if let (Some(second), None) = (second.as_ref(), third.as_ref()) {
      write!(f, " {} {}", self.conjunction, second)?;
      return Ok(());
    }

    let mut current = second;
    let mut next = third;

    loop {
      match (current, next) {
        (Some(c), Some(n)) => {
          write!(f, ", {}", c)?;
          current = Some(n);
          next = values.next();
        }
        (Some(c), None) => {
          write!(f, ", {} {}", self.conjunction, c)?;
          return Ok(());
        }
        _ => panic!("Iterator was fused, but returned Some after None"),
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn or() {
    assert_eq!("1", List::or(&[1]).to_string());
    assert_eq!("1 or 2", List::or(&[1, 2]).to_string());
    assert_eq!("1, 2, or 3", List::or(&[1, 2, 3]).to_string());
    assert_eq!("1, 2, 3, or 4", List::or(&[1, 2, 3, 4]).to_string());
  }

  #[test]
  fn and() {
    assert_eq!("1", List::and(&[1]).to_string());
    assert_eq!("1 and 2", List::and(&[1, 2]).to_string());
    assert_eq!("1, 2, and 3", List::and(&[1, 2, 3]).to_string());
    assert_eq!("1, 2, 3, and 4", List::and(&[1, 2, 3, 4]).to_string());
  }

  #[test]
  fn or_ticked() {
    assert_eq!("`1`", List::or_ticked(&[1]).to_string());
    assert_eq!("`1` or `2`", List::or_ticked(&[1, 2]).to_string());
    assert_eq!("`1`, `2`, or `3`", List::or_ticked(&[1, 2, 3]).to_string());
    assert_eq!(
      "`1`, `2`, `3`, or `4`",
      List::or_ticked(&[1, 2, 3, 4]).to_string()
    );
  }

  #[test]
  fn and_ticked() {
    assert_eq!("`1`", List::and_ticked(&[1]).to_string());
    assert_eq!("`1` and `2`", List::and_ticked(&[1, 2]).to_string());
    assert_eq!(
      "`1`, `2`, and `3`",
      List::and_ticked(&[1, 2, 3]).to_string()
    );
    assert_eq!(
      "`1`, `2`, `3`, and `4`",
      List::and_ticked(&[1, 2, 3, 4]).to_string()
    );
  }
}
