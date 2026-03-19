use super::*;

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq)]
pub(crate) enum Shell {
  Bash,
  Elvish,
  Fish,
  #[value(alias = "nu")]
  Nushell,
  Powershell,
  Zsh,
}

const NUSHELL_COMPLETION_SCRIPT: &str = r#"def "nu-complete just" [] {
    (^just --dump --unstable --dump-format json | from json).recipes | transpose recipe data | flatten | where {|row| $row.private == false } | select recipe doc parameters | rename value description
}

# Just: A Command Runner
export extern "just" [
    ...recipe: string@"nu-complete just", # Recipe(s) to run, may be with argument(s)
]
"#;

impl Shell {
  pub(crate) fn completion_script(self) -> String {
    let shell = match self {
      Self::Bash => "bash",
      Self::Elvish => "elvish",
      Self::Fish => "fish",
      Self::Powershell => "powershell",
      Self::Zsh => "zsh",
      Self::Nushell => return NUSHELL_COMPLETION_SCRIPT.into(),
    };

    // don't swallow this error
    let completer = env::current_exe()
      .ok()
      .map_or_else(|| "just".into(), |p| p.to_string_lossy().into_owned());

    let mut buffer = Vec::new();

    clap_complete::env::Shells::builtins()
      .completer(shell)
      .unwrap()
      .write_registration("JUST_COMPLETE", "just", "just", &completer, &mut buffer)
      .unwrap();

    String::from_utf8(buffer).unwrap()
  }
}
