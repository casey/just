use std::process::Command;

pub enum Shell<'a> {
  ShLike(&'a str),
  Custom(&'a str),
}

impl<'a> Shell<'a> {
  pub fn new(cmd: &str) -> Shell {
    //! Build a Shell variant from a string (shell executable name)
    match cmd {
      "sh" | "bash" | "dash" => Shell::ShLike(cmd),
      cmd => Shell::Custom(cmd),
    }
  }
  pub fn command(&self) -> Command {
    //! Return a `Command` instance with the right executable and arguments
    match self {
      Shell::ShLike(s) => {
        let mut cmd = Command::new(s);
        cmd.arg("-cu");
        cmd
      }
      Shell::Custom(s) => Command::new(s),
    }
  }
  pub fn name(&self) -> &str {
    match self {
      Shell::ShLike(s) => s,
      Shell::Custom(cmd) => cmd,
    }
  }
}
