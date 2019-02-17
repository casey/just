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
fn test_shell_wincmd() {
  let shell = Shell::new("cmd");
  let output = shell
    .command()
    .arg("echo Test from cmd")
    .output()
    .expect("invocation failed");
  let stdout = std::str::from_utf8(&output.stdout).expect("Unable to decode stdout");
  assert_eq!(stdout, "Test from cmd\r\n");
}

#[test]
fn test_shell_bash() {
  let shell = Shell::new("bash");
  let output = shell
    .command()
    .arg("echo Test from bash")
    .output()
    .expect("invocation failed");
  let stdout = std::str::from_utf8(&output.stdout).expect("Unable to decode stdout");
  assert_eq!(stdout, "Test from bash\n");
}
