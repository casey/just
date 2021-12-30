use crate::common::*;

pub(crate) const DEFAULT_SHELL: &str = "sh";
pub(crate) const DEFAULT_SHELL_ARG: &str = "-cu";
pub(crate) const WINDOWS_DEFAULT_SHELL: &str = "powershell.exe";
pub(crate) const WINDOWS_DEFAULT_SHELL_ARG: &str = "-c";

#[derive(Debug, PartialEq, Serialize)]
pub(crate) struct Settings<'src> {
  pub(crate) dotenv_load: Option<bool>,
  pub(crate) export: bool,
  pub(crate) positional_arguments: bool,
  pub(crate) shell: Option<setting::Shell<'src>>,
  pub(crate) windows_powershell: bool,
}

impl<'src> Settings<'src> {
  pub(crate) fn new() -> Settings<'src> {
    Settings {
      dotenv_load: None,
      export: false,
      positional_arguments: false,
      shell: None,
      windows_powershell: false,
    }
  }

  pub(crate) fn shell_command(&self, config: &Config) -> Command {
    let mut cmd = Command::new(self.shell_binary(config));

    cmd.args(self.shell_arguments(config));

    cmd
  }

  pub(crate) fn shell_binary<'a>(&'a self, config: &'a Config) -> &'a str {
    if config.shell_present {
      return &config.shell;
    }

    if cfg!(windows) && self.windows_powershell {
      return WINDOWS_DEFAULT_SHELL;
    }

    if let Some(shell) = &self.shell {
      return shell.command.cooked.as_ref();
    } else {
      return DEFAULT_SHELL;
    }
  }

  pub(crate) fn shell_arguments<'a>(&'a self, config: &'a Config) -> Vec<&'a str> {
    if config.shell_present {
      return config.shell_args.iter().map(String::as_ref).collect();
    }

    if cfg!(windows) && self.windows_powershell {
      return vec![WINDOWS_DEFAULT_SHELL_ARG];
    }

    if let Some(shell) = &self.shell {
      return shell
        .arguments
        .iter()
        .map(|argument| argument.cooked.as_ref())
        .collect();
    } else {
      return vec![DEFAULT_SHELL_ARG];
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::setting::Shell;

  use super::*;

  #[test]
  fn default_shell() {
    let mut settings = Settings::new();
    settings.windows_powershell = false;

    let config = Config {
      shell_present: false,
      shell_command: false,
      ..testing::config(&[])
    };

    assert_eq!(settings.shell_binary(&config), "sh");
    assert_eq!(settings.shell_arguments(&config), vec!["-cu"]);
  }

  #[test]
  fn default_shell_powershell() {
    let mut settings = Settings::new();
    settings.windows_powershell = true;

    let config = Config {
      shell_present: false,
      shell_command: false,
      ..testing::config(&[])
    };

    if cfg!(windows) {
      assert_eq!(settings.shell_binary(&config), "powershell.exe");
      assert_eq!(settings.shell_arguments(&config), vec!["-c"]);
    } else {
      assert_eq!(settings.shell_binary(&config), "sh");
      assert_eq!(settings.shell_arguments(&config), vec!["-cu"]);
    }
  }

  #[test]
  fn overwrite_shell() {
    let mut settings = Settings::new();
    settings.windows_powershell = false;

    let config = Config {
      shell_present: true,
      shell_command: true,
      shell: "lol".to_string(),
      shell_args: vec!["-nice".to_string()],
      ..testing::config(&[])
    };

    assert_eq!(settings.shell_binary(&config), "lol");
    assert_eq!(settings.shell_arguments(&config), vec!["-nice"]);
  }

  #[test]
  fn overwrite_shell_powershell() {
    let mut settings = Settings::new();
    settings.windows_powershell = true;

    let config = Config {
      shell_present: true,
      shell_command: true,
      shell: "lol".to_string(),
      shell_args: vec!["-nice".to_string()],
      ..testing::config(&[])
    };

    assert_eq!(settings.shell_binary(&config), "lol");
    assert_eq!(settings.shell_arguments(&config), vec!["-nice"]);
  }

  #[test]
  fn shell_cooked() {
    let mut settings = Settings::new();
    settings.windows_powershell = false;

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
      shell_present: false,
      shell_command: false,
      ..testing::config(&[])
    };

    assert_eq!(settings.shell_binary(&config), "asdf.exe");
    assert_eq!(settings.shell_arguments(&config), vec!["-nope"]);
  }
}
