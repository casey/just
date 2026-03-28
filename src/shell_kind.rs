pub(crate) enum ShellKind {
  Cmd,
  Other,
  Powershell,
  Pwsh,
}

impl ShellKind {
  pub(crate) fn extension(self) -> &'static str {
    match self {
      Self::Cmd => ".bat",
      Self::Powershell | Self::Pwsh => ".ps1",
      Self::Other => "",
    }
  }

  pub(crate) fn takes_shell_name(self) -> bool {
    match self {
      Self::Cmd | Self::Powershell | Self::Pwsh => false,
      Self::Other => true,
    }
  }
}

impl From<&str> for ShellKind {
  fn from(command: &str) -> Self {
    match command {
      "cmd" | "cmd.exe" => Self::Cmd,
      "powershell" | "powershell.exe" => Self::Powershell,
      "pwsh" | "pwsh.exe" => Self::Pwsh,
      _ => Self::Other,
    }
  }
}
