use super::*;

#[derive(Clone, Copy)]
pub(crate) enum ShellKind {
  Cmd,
  Other,
  Powershell,
}

impl ShellKind {
  pub(crate) fn extension(self) -> &'static str {
    match self {
      Self::Cmd => ".bat",
      Self::Powershell => ".ps1",
      Self::Other => "",
    }
  }

  pub(crate) fn takes_shell_name(self) -> bool {
    match self {
      Self::Cmd | Self::Powershell => false,
      Self::Other => true,
    }
  }
}

impl From<&str> for ShellKind {
  fn from(command: &str) -> Self {
    match command {
      "cmd" | "cmd.exe" => Self::Cmd,
      "powershell" | "powershell.exe" | "pwsh" | "pwsh.exe" => Self::Powershell,
      _ => Self::Other,
    }
  }
}

impl From<&Command> for ShellKind {
  fn from(command: &Command) -> Self {
    let command = Path::new(command.get_program());

    let Some(command) = command.file_name() else {
      return Self::Other;
    };

    let Some(command) = command.to_str() else {
      return Self::Other;
    };

    command.into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_str() {
    #[track_caller]
    fn case(s: &str, takes_shell_name: bool, extension: &str) {
      let kind = ShellKind::from(s);
      assert_eq!(
        kind.takes_shell_name(),
        takes_shell_name,
        "takes_shell_name for {s:?}"
      );
      assert_eq!(kind.extension(), extension, "extension for {s:?}");
    }

    case("foo", true, "");
    case("cmd", false, ".bat");
    case("cmd.exe", false, ".bat");
    case("powershell", false, ".ps1");
    case("powershell.exe", false, ".ps1");
    case("pwsh", false, ".ps1");
    case("pwsh.exe", false, ".ps1");
  }
}
