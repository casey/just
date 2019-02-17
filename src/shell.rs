use std::process::Command;

pub enum Shell<'a> {
  ShLike(&'a str),
  WinCmd(&'a str),
  Custom(&'a str),
}

impl<'a> Shell<'a> {
  pub fn new(cmd: &str) -> Shell {
    //! Build a Shell variant from a string (shell executable name)
    match cmd {
      "sh" | "bash" | "dash" => Shell::ShLike(cmd),
      "cmd" | "cmd.exe" => Shell::WinCmd(cmd),
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
      Shell::WinCmd(s) => {
        let mut cmd = Command::new(s);
        cmd.arg("/C");
        cmd
      }
      Shell::Custom(s) => Command::new(s),
    }
  }
  pub fn name(&self) -> &str {
    match self {
      Shell::ShLike(s) => s,
      Shell::WinCmd(s) => s,
      Shell::Custom(cmd) => cmd,
    }
  }
}

#[cfg(windows)]
#[test]
fn test_wincmd() {
  let output = Shell::new("cmd")
    .command()
    .arg("echo Test cmd.exe")
    .output()
    .expect("cmd invocation failed");
  let stdout = str::from_utf8(&output.stdout).unwrap()
  assert_eq!(stdout, "Test cmd.exe");
}
