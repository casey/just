use super::*;

pub(crate) trait RangeExt<T> {
  fn display(&self) -> DisplayRange<&Self> {
    DisplayRange(self)
  }

  fn range_contains(&self, i: &T) -> bool;
}

pub(crate) struct DisplayRange<T>(T);

impl Display for DisplayRange<&RangeInclusive<usize>> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.0.start() == self.0.end() {
      write!(f, "{}", self.0.start())?;
    } else if *self.0.end() == usize::MAX {
      write!(f, "{} or more", self.0.start())?;
    } else {
      write!(f, "{} to {}", self.0.start(), self.0.end())?;
    }
    Ok(())
  }
}

impl<T> RangeExt<T> for Range<T>
where
  T: PartialOrd,
{
  fn range_contains(&self, i: &T) -> bool {
    i >= &self.start && i < &self.end
  }
}

impl<T> RangeExt<T> for RangeInclusive<T>
where
  T: PartialOrd,
{
  fn range_contains(&self, i: &T) -> bool {
    i >= self.start() && i <= self.end()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn exclusive() {
    assert!(!(0..0).range_contains(&0));
    assert!(!(1..10).range_contains(&0));
    assert!(!(1..10).range_contains(&10));
    assert!(!(1..10).range_contains(&0));
    assert!((0..1).range_contains(&0));
    assert!((10..20).range_contains(&15));
  }

  #[test]
  fn inclusive() {
    assert!(!(0..=10).range_contains(&11));
    assert!(!(1..=10).range_contains(&0));
    assert!(!(5..=10).range_contains(&4));
    assert!((0..=0).range_contains(&0));
    assert!((0..=1).range_contains(&0));
    assert!((0..=10).range_contains(&0));
    assert!((0..=10).range_contains(&10));
    assert!((0..=10).range_contains(&7));
    assert!((1..=10).range_contains(&10));
    assert!((10..=20).range_contains(&15));
  }

  #[test]
  fn display() {
    assert!(!(1..1).contains(&1));
    assert!((1..1).is_empty());
    assert!((5..5).is_empty());
    assert_eq!((0..=0).display().to_string(), "0");
    assert_eq!((1..=1).display().to_string(), "1");
    assert_eq!((5..=5).display().to_string(), "5");
    assert_eq!((5..=9).display().to_string(), "5 to 9");
    assert_eq!((1..=usize::MAX).display().to_string(), "1 or more");
  }
}
