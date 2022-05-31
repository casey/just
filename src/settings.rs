use crate::common::*;

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
    let mut cmd = Command::new(self.shell_binary(config));

    cmd.args(self.shell_arguments(config));

    cmd
  }

  pub(crate) fn shell_binary<'a>(&'a self, config: &'a Config) -> &'a str {
    let shell_or_args_present = config.shell.is_some() || config.shell_args.is_some();

    if let (Some(shell), false) = (&self.shell, shell_or_args_present) {
      shell.command.cooked.as_ref()
    } else if let Some(shell) = &config.shell {
      shell
    } else if let (true, Some(shell)) = (cfg!(windows), &self.windows_shell) {
      shell.command.cooked.as_ref()
    } else if cfg!(windows) && self.windows_powershell {
      WINDOWS_POWERSHELL_SHELL
    } else {
      DEFAULT_SHELL
    }
  }

  pub(crate) fn shell_arguments<'a>(&'a self, config: &'a Config) -> Vec<&'a str> {
    let shell_or_args_present = config.shell.is_some() || config.shell_args.is_some();

    if let (Some(shell), false) = (&self.shell, shell_or_args_present) {
      shell
        .arguments
        .iter()
        .map(|argument| argument.cooked.as_ref())
        .collect()
    } else if let Some(shell_args) = &config.shell_args {
      shell_args.iter().map(String::as_ref).collect()
    } else if let (true, Some(shell)) = (cfg!(windows), &self.windows_shell) {
      shell
        .arguments
        .iter()
        .map(|argument| argument.cooked.as_ref())
        .collect()
    } else if cfg!(windows) && self.windows_powershell {
      WINDOWS_POWERSHELL_ARGS.to_vec()
    } else {
      DEFAULT_SHELL_ARGS.to_vec()
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

    assert_eq!(settings.shell_binary(&config), "sh");
    assert_eq!(settings.shell_arguments(&config), vec!["-cu"]);
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
      assert_eq!(settings.shell_binary(&config), "powershell.exe");
      assert_eq!(
        settings.shell_arguments(&config),
        vec!["-NoLogo", "-Command"]
      );
    } else {
      assert_eq!(settings.shell_binary(&config), "sh");
      assert_eq!(settings.shell_arguments(&config), vec!["-cu"]);
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

    assert_eq!(settings.shell_binary(&config), "lol");
    assert_eq!(settings.shell_arguments(&config), vec!["-nice"]);
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

    assert_eq!(settings.shell_binary(&config), "lol");
    assert_eq!(settings.shell_arguments(&config), vec!["-nice"]);
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

    assert_eq!(settings.shell_binary(&config), "asdf.exe");
    assert_eq!(settings.shell_arguments(&config), vec!["-nope"]);
  }

  #[test]
  fn shell_present_but_not_shell_args() {
    let mut settings = Settings::new();
    settings.windows_powershell = true;

    let config = Config {
      shell: Some("lol".to_string()),
      ..testing::config(&[])
    };

    assert_eq!(settings.shell_binary(&config), "lol");
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
      assert_eq!(settings.shell_binary(&config), "powershell.exe");
    } else {
      assert_eq!(settings.shell_binary(&config), "sh");
    }

    assert_eq!(settings.shell_arguments(&config), vec!["-nice"]);
  }
}
