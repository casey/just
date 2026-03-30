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
