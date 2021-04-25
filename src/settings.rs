use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Settings<'src> {
  pub(crate) dotenv_load:          Option<bool>,
  pub(crate) export:               bool,
  pub(crate) positional_arguments: bool,
  pub(crate) shell:                Option<setting::Shell<'src>>,
}

impl<'src> Settings<'src> {
  pub(crate) fn new() -> Settings<'src> {
    Settings {
      dotenv_load:          None,
      export:               false,
      positional_arguments: false,
      shell:                None,
    }
  }

  pub(crate) fn shell_command(&self, config: &Config) -> Command {
    let mut cmd = Command::new(self.shell_binary(config));

    cmd.args(self.shell_arguments(config));

    cmd
  }

  pub(crate) fn shell_binary<'a>(&'a self, config: &'a Config) -> &'a str {
    if let (Some(shell), false) = (&self.shell, config.shell_present) {
      shell.command.cooked.as_ref()
    } else {
      &config.shell
    }
  }

  pub(crate) fn shell_arguments<'a>(&'a self, config: &'a Config) -> Vec<&'a str> {
    if let (Some(shell), false) = (&self.shell, config.shell_present) {
      shell
        .arguments
        .iter()
        .map(|argument| argument.cooked.as_ref())
        .collect()
    } else {
      config.shell_args.iter().map(String::as_ref).collect()
    }
  }
}
