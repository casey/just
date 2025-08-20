use super::*;

pub(crate) trait RangeExt<T> {
  fn display(&self) -> DisplayRange<&Self> {
    DisplayRange(self)
  }
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

impl<T> RangeExt<T> for Range<T> where T: PartialOrd {}

impl<T> RangeExt<T> for RangeInclusive<T> where T: PartialOrd {}

#[cfg(test)]
mod tests {
  use super::*;

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
