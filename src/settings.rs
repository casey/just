use super::*;

pub(crate) const DEFAULT_SHELL: &str = "sh";
pub(crate) const DEFAULT_SHELL_ARGS: &[&str] = &["-cu"];
pub(crate) const WINDOWS_POWERSHELL_SHELL: &str = "powershell.exe";
pub(crate) const WINDOWS_POWERSHELL_ARGS: &[&str] = &["-NoLogo", "-Command"];

#[derive(Debug, PartialEq, Serialize)]
pub(crate) struct Settings<'src> {
  pub(crate) allow_duplicate_recipes: bool,
  pub(crate) dotenv_load: Option<bool>,
  pub(crate) export: bool,
  pub(crate) positional_arguments: bool,
  pub(crate) shell: Option<Shell<'src>>,
  pub(crate) windows_powershell: bool,
  pub(crate) windows_shell: Option<Shell<'src>>,
}

impl<'src> Settings<'src> {
  pub(crate) fn new() -> Settings<'src> {
    Settings {
      allow_duplicate_recipes: false,
      dotenv_load: None,
      export: false,
      positional_arguments: false,
      shell: None,
      windows_powershell: false,
      windows_shell: None,
    }
  }

  pub(crate) fn shell_command(&self, config: &Config) -> Command {
    let (command, args) = self.shell(config);

    let mut cmd = Command::new(command);

    cmd.args(args);

    cmd
  }

  pub(crate) fn shell<'a>(&'a self, config: &'a Config) -> (&'a str, Vec<&'a str>) {
    match (&config.shell, &config.shell_args) {
      (Some(shell), Some(shell_args)) => {
        return (shell, shell_args.iter().map(String::as_ref).collect())
      }
      (Some(shell), None) => return (shell, DEFAULT_SHELL_ARGS.to_vec()),
      (None, Some(shell_args)) => {
        return (
          DEFAULT_SHELL,
          shell_args.iter().map(String::as_ref).collect(),
        )
      }
      (None, None) => {}
    }

    if let (true, Some(shell)) = (cfg!(windows), &self.windows_shell) {
      (
        shell.command.cooked.as_ref(),
        shell
          .arguments
          .iter()
          .map(|argument| argument.cooked.as_ref())
          .collect(),
      )
    } else if cfg!(windows) && self.windows_powershell {
      (WINDOWS_POWERSHELL_SHELL, WINDOWS_POWERSHELL_ARGS.to_vec())
    } else if let Some(shell) = &self.shell {
      (
        shell.command.cooked.as_ref(),
        shell
          .arguments
          .iter()
          .map(|argument| argument.cooked.as_ref())
          .collect(),
      )
    } else {
      (DEFAULT_SHELL, DEFAULT_SHELL_ARGS.to_vec())
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_shell() {
    let settings = Settings::new();

    let config = Config {
      shell_command: false,
      ..testing::config(&[])
    };

    assert_eq!(settings.shell(&config), ("sh", vec!["-cu"]));
  }

  #[test]
  fn default_shell_powershell() {
    let mut settings = Settings::new();
    settings.windows_powershell = true;

    let config = Config {
      shell_command: false,
      ..testing::config(&[])
    };

    if cfg!(windows) {
      assert_eq!(
        settings.shell(&config),
        ("powershell.exe", vec!["-NoLogo", "-Command"])
      );
    } else {
      assert_eq!(settings.shell(&config), ("sh", vec!["-cu"]));
    }
  }

  #[test]
  fn overwrite_shell() {
    let settings = Settings::new();

    let config = Config {
      shell_command: true,
      shell: Some("lol".to_string()),
      shell_args: Some(vec!["-nice".to_string()]),
      ..testing::config(&[])
    };

    assert_eq!(settings.shell(&config), ("lol", vec!["-nice"]));
  }

  #[test]
  fn overwrite_shell_powershell() {
    let mut settings = Settings::new();
    settings.windows_powershell = true;

    let config = Config {
      shell_command: true,
      shell: Some("lol".to_string()),
      shell_args: Some(vec!["-nice".to_string()]),
      ..testing::config(&[])
    };

    assert_eq!(settings.shell(&config), ("lol", vec!["-nice"]));
  }

  #[test]
  fn shell_cooked() {
    let mut settings = Settings::new();

    settings.shell = Some(Shell {
      command: StringLiteral {
        kind: StringKind::from_token_start("\"").unwrap(),
        raw: "asdf.exe",
        cooked: "asdf.exe".to_string(),
      },
      arguments: vec![StringLiteral {
        kind: StringKind::from_token_start("\"").unwrap(),
        raw: "-nope",
        cooked: "-nope".to_string(),
      }],
    });

    let config = Config {
      shell_command: false,
      ..testing::config(&[])
    };

    assert_eq!(settings.shell(&config), ("asdf.exe", vec!["-nope"]));
  }

  #[test]
  fn shell_present_but_not_shell_args() {
    let mut settings = Settings::new();
    settings.windows_powershell = true;

    let config = Config {
      shell: Some("lol".to_string()),
      ..testing::config(&[])
    };

    assert_eq!(settings.shell(&config).0, "lol");
  }

  #[test]
  fn shell_args_present_but_not_shell() {
    let mut settings = Settings::new();
    settings.windows_powershell = true;

    let config = Config {
      shell_command: false,
      shell_args: Some(vec!["-nice".to_string()]),
      ..testing::config(&[])
    };

    if cfg!(windows) {
      assert_eq!(settings.shell(&config), ("powershell.exe", vec!["-nice"]));
    } else {
      assert_eq!(settings.shell(&config), ("sh", vec!["-nice"]));
    }
  }
}
