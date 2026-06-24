use super::*;

const N: &str = "(0|[1-9][0-9]{0,8})";

static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(&format!(r"^{N}\.{N}\.{N}$")).unwrap());

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Version(u64, u64, u64);

impl Version {
  pub(crate) fn current() -> Self {
    VERSION.parse().unwrap()
  }
}

impl FromStr for Version {
  type Err = Box<dyn std::error::Error>;

  fn from_str(text: &str) -> Result<Self, Self::Err> {
    let captures = REGEX
      .captures(text)
      .ok_or_else(|| format!("expected `MAJOR.MINOR.PATCH` version, but found `{text}`"))?;

    Ok(Version(
      captures[1].parse().unwrap(),
      captures[2].parse().unwrap(),
      captures[3].parse().unwrap(),
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn current_version_is_valid() {
    Version::current();
  }

  #[test]
  fn parse() {
    #[track_caller]
    fn case(text: &str, expected: Option<Version>) {
      assert_eq!(text.parse::<Version>().ok(), expected);
    }

    case("1.2.3", Some(Version(1, 2, 3)));
    case("0.0.0", Some(Version(0, 0, 0)));
    case("", None);
    case("foo", None);
    case("1", None);
    case("1.2", None);
    case("1.2.3.4", None);
    case("1.2.x", None);
    case("+1.2.3", None);
    case("01.2.3", None);
    case("999999999.0.0", Some(Version(999_999_999, 0, 0)));
    case("1234567890.0.0", None);
  }

  #[test]
  fn compare() {
    #[track_caller]
    fn case(a: &str, b: &str, expected: Ordering) {
      assert_eq!(
        a.parse::<Version>()
          .unwrap()
          .cmp(&b.parse::<Version>().unwrap()),
        expected
      );
    }

    case("1.2.3", "1.2.3", Ordering::Equal);
    case("1.2.3", "1.2.4", Ordering::Less);
    case("1.3.0", "1.2.9", Ordering::Greater);
    case("2.0.0", "1.99.99", Ordering::Greater);
  }
}
