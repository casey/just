use super::*;

pub(crate) trait RangeExt<T> {
  fn range_contains(&self, i: &T) -> bool;

  fn display(&self) -> DisplayRange<&Self> {
    DisplayRange(self)
  }
}

pub(crate) struct DisplayRange<T>(T);

impl Display for DisplayRange<&Range<usize>> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.0.start == self.0.end {
      write!(f, "0")?;
    } else if self.0.start == self.0.end - 1 {
      write!(f, "{}", self.0.start)?;
    } else if self.0.end == usize::MAX {
      write!(f, "{} or more", self.0.start)?;
    } else {
      // the range is exclusive from above so it is "start to end-1"
      write!(f, "{} to {}", self.0.start, self.0.end - 1)?;
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
    assert!((1..1).len() == 0);
    assert!((5..5).len() == 0);
    assert_eq!((1..1).display().to_string(), "0");
    assert_eq!((1..2).display().to_string(), "1");
    assert_eq!((1..usize::MAX).display().to_string(), "1 or more");
  }
}
