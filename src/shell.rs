use super::*;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum, EnumIter)]
pub(crate) enum Shell {
  Bash,
  Elvish,
  Fish,
  #[value(alias = "nu")]
  Nushell,
  Powershell,
  Zsh,
}

impl Shell {
  pub(crate) fn script(self) -> &'static str {
    match self {
      Self::Bash => include_str!("../completions/just.bash"),
      Self::Elvish => include_str!("../completions/just.elvish"),
      Self::Fish => include_str!("../completions/just.fish"),
      Self::Nushell => include_str!("../completions/just.nu"),
      Self::Powershell => include_str!("../completions/just.powershell"),
      Self::Zsh => include_str!("../completions/just.zsh"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn completion_scripts() {
    for shell in Shell::iter() {
      if shell == Shell::Nushell {
        continue;
      }

      assert!(
        shell
          .script()
          .contains(Arguments::COMPLETION_ENVIRONMENT_VARIABLE),
        "shell {shell:?} does not contain `{}`",
        Arguments::COMPLETION_ENVIRONMENT_VARIABLE,
      );
    }
  }
}
