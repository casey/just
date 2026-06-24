use super::*;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Version(Vec<u64>);

impl Version {
  pub(crate) fn parse(text: &str) -> Option<Version> {
    text
      .split('.')
      .map(|component| component.parse::<u64>().ok())
      .collect::<Option<Vec<u64>>>()
      .map(Version)
  }
}

impl Ord for Version {
  fn cmp(&self, other: &Self) -> Ordering {
    for i in 0..self.0.len().max(other.0.len()) {
      let a = self.0.get(i).copied().unwrap_or(0);
      let b = other.0.get(i).copied().unwrap_or(0);
      match a.cmp(&b) {
        Ordering::Equal => {}
        ordering => return ordering,
      }
    }
    Ordering::Equal
  }
}

impl PartialOrd for Version {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    #[track_caller]
    fn case(text: &str, expected: Option<&[u64]>) {
      assert_eq!(
        Version::parse(text),
        expected.map(|components| Version(components.to_vec()))
      );
    }

    case("1", Some(&[1]));
    case("1.2", Some(&[1, 2]));
    case("1.2.3", Some(&[1, 2, 3]));
    case("", None);
    case("foo", None);
    case("1.", None);
    case("1.x", None);
  }

  #[test]
  fn compare() {
    #[track_caller]
    fn case(a: &str, b: &str, expected: Ordering) {
      assert_eq!(
        Version::parse(a).unwrap().cmp(&Version::parse(b).unwrap()),
        expected
      );
    }

    case("1.2.3", "1.2.3", Ordering::Equal);
    case("1.2", "1.2.0", Ordering::Equal);
    case("1.2.0", "1.2", Ordering::Equal);
    case("1.2.3", "1.2.4", Ordering::Less);
    case("1.3.0", "1.2.9", Ordering::Greater);
    case("2", "1.99.99", Ordering::Greater);
  }
}
