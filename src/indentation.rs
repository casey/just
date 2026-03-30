use super::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Indentation {
  character: char,
  count: usize,
}

impl Default for Indentation {
  fn default() -> Self {
    Self {
      character: ' ',
      count: 4,
    }
  }
}

impl Display for Indentation {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    for _ in 0..self.count {
      write!(f, "{}", self.character)?;
    }
    Ok(())
  }
}

impl FromStr for Indentation {
  type Err = &'static str;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut chars = s.chars();

    let Some(character) = chars.next() else {
      return Err("indentation must not be empty");
    };

    if !matches!(character, ' ' | '\t') {
      return Err("indentation must be spaces or tabs");
    }

    if !chars.all(|c| c == character) {
      return Err("indentation may not be mixed");
    }

    Ok(Self {
      count: s.chars().count(),
      character,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_str() {
    #[track_caller]
    fn case(s: &str, character: char, count: usize) {
      assert_eq!(
        s.parse::<Indentation>().unwrap(),
        Indentation { character, count },
      );
    }

    case("    ", ' ', 4);
    case("  ", ' ', 2);
    case("\t", '\t', 1);
    case("\t\t", '\t', 2);
  }

  #[test]
  fn from_str_error() {
    #[track_caller]
    fn case(s: &str, expected: &str) {
      assert_eq!(s.parse::<Indentation>().unwrap_err(), expected);
    }

    case("", "indentation must not be empty");
    case("x", "indentation must be spaces or tabs");
    case(" \t", "indentation may not be mixed");
  }
}
