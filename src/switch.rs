use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Switch {
  Long(String),
  Short(char),
}

impl Switch {
  pub(crate) fn apply<'src>(
    self,
    recipe: &Recipe<'src>,
    long: &BTreeMap<&str, usize>,
    short: &BTreeMap<char, usize>,
    arguments: &mut [Value],
    rest: &[&str],
    i: &mut usize,
    value: Option<&str>,
    last: bool,
  ) -> RunResult<'src> {
    let index = match &self {
      Self::Long(name) => long.get(name.as_str()),
      Self::Short(name) => short.get(name),
    };

    let Some(&index) = index else {
      return Err(Error::UnknownOption {
        recipe: recipe.name(),
        switch: self,
      });
    };

    let parameter = &recipe.parameters[index];

    let value = if parameter.flag || parameter.value.is_some() {
      if value.is_some() {
        return Err(Error::FlagWithValue {
          recipe: recipe.name(),
          switch: self,
        });
      }
      "true"
    } else if !last {
      return Err(Error::NonFinalOptionWithValue {
        recipe: recipe.name(),
        switch: self,
      });
    } else if let Some(value) = value {
      value
    } else {
      let Some(&value) = rest.get(*i + 1) else {
        return Err(Error::OptionMissingValue {
          recipe: recipe.name(),
          switch: self,
        });
      };
      *i += 1;
      value
    };

    let max = if let Some(bound) = &parameter.bound {
      bound.max
    } else if parameter.kind.is_variadic() {
      None
    } else {
      Some(1)
    };

    let group = &mut arguments[index];

    if let Some(max) = max
      && group.elements().len() >= max
    {
      return Err(if max == 1 {
        Error::DuplicateOption {
          recipe: recipe.name(),
          switch: self,
        }
      } else {
        Error::OptionAboveMaximum {
          recipe: recipe.name(),
          switch: self,
          max,
          found: group.elements().len() + 1,
        }
      });
    }

    group.push(value);

    Ok(())
  }
}

impl Display for Switch {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match &self {
      Self::Long(long) => write!(f, "--{long}"),
      Self::Short(short) => write!(f, "-{short}"),
    }
  }
}
