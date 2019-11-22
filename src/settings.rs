use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Settings<'src> {
  pub(crate) shell: Option<setting::Shell<'src>>,
}

impl<'src> Settings<'src> {
  pub(crate) fn new() -> Settings<'src> {
    Settings { shell: None }
  }

  pub(crate) fn shell_command(&self, config: &Config) -> Command {
    if let (Some(shell), false) = (&self.shell, config.shell_present) {
      let mut cmd = Command::new(shell.command.cooked.as_ref());

      for argument in &shell.arguments {
        cmd.arg(argument.cooked.as_ref());
      }

      cmd
    } else {
      let mut cmd = Command::new(&config.shell);

      cmd.args(&config.shell_args);

      cmd
    }
  }
}
