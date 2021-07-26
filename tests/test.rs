use crate::common::*;

macro_rules! test {
  (
    name:       $name:ident,
    $(justfile: $justfile:expr,)?
    $(args:     ($($arg:tt),*),)?
    $(env:      { $($env_key:literal : $env_value:literal,)* },)?
    $(stdin:    $stdin:expr,)?
    $(stdout:   $stdout:expr,)?
    $(stderr:   $stderr:expr,)?
    $(status:   $status:expr,)?
    $(shell:    $shell:expr,)?
  ) => {
    #[test]
    fn $name() {
      let test = crate::test::Test::new();

      $($(let test = test.arg($arg);)*)?
      $($(let test = test.env($env_key, $env_value);)*)?
      $(let test = test.justfile($justfile);)?
      $(let test = test.shell($shell);)?
      $(let test = test.status($status);)?
      $(let test = test.stderr($stderr);)?
      $(let test = test.stdin($stdin);)?
      $(let test = test.stdout($stdout);)?

      test.run()
    }
  }
}

pub(crate) struct Test {
  pub(crate) directory: TempDir,
  pub(crate) justfile:  Option<String>,
  pub(crate) args:      Vec<String>,
  pub(crate) env:       BTreeMap<String, String>,
  pub(crate) stdin:     String,
  pub(crate) stdout:    String,
  pub(crate) stderr:    String,
  pub(crate) status:    i32,
  pub(crate) shell:     bool,
}

impl Test {
  pub(crate) fn new() -> Self {
    Self {
      args:      Vec::new(),
      directory: tempdir(),
      env:       BTreeMap::new(),
      justfile:  Some(String::new()),
      shell:     true,
      status:    EXIT_SUCCESS,
      stderr:    String::new(),
      stdin:     String::new(),
      stdout:    String::new(),
    }
  }

  pub(crate) fn arg(mut self, val: &str) -> Self {
    self.args.push(val.to_owned());
    self
  }

  pub(crate) fn env(mut self, key: &str, val: &str) -> Self {
    self.env.insert(key.to_string(), val.to_string());
    self
  }

  pub(crate) fn justfile(mut self, justfile: impl Into<String>) -> Self {
    self.justfile = Some(justfile.into());
    self
  }

  pub(crate) fn justfile_path(&self) -> PathBuf {
    self.directory.path().join("justfile")
  }

  pub(crate) fn no_justfile(mut self) -> Self {
    self.justfile = None;
    self
  }

  pub(crate) fn shell(mut self, shell: bool) -> Self {
    self.shell = shell;
    self
  }

  pub(crate) fn status(mut self, exit_status: i32) -> Self {
    self.status = exit_status;
    self
  }

  pub(crate) fn stderr(mut self, stderr: impl Into<String>) -> Self {
    self.stderr = stderr.into();
    self
  }

  pub(crate) fn stdin(mut self, stdin: impl Into<String>) -> Self {
    self.stdin = stdin.into();
    self
  }

  pub(crate) fn stdout(mut self, stdout: impl Into<String>) -> Self {
    self.stdout = stdout.into();
    self
  }

  pub(crate) fn args(mut self, args: &[&str]) -> Self {
    for arg in args {
      self = self.arg(arg);
    }
    self
  }
}

impl Test {
  pub(crate) fn run(self) {
    if let Some(justfile) = &self.justfile {
      let justfile = unindent(justfile);
      fs::write(self.justfile_path(), justfile).unwrap();
    }

    let stdout = unindent(&self.stdout);
    let stderr = unindent(&self.stderr);

    let mut dotenv_path = self.directory.path().to_path_buf();
    dotenv_path.push(".env");
    fs::write(dotenv_path, "DOTENV_KEY=dotenv-value").unwrap();

    let mut command = Command::new(&executable_path("just"));

    if self.shell {
      command.args(&["--shell", "bash"]);
    }

    let mut child = command
      .args(self.args)
      .envs(&self.env)
      .current_dir(self.directory.path())
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .expect("just invocation failed");

    {
      let mut stdin_handle = child.stdin.take().expect("failed to unwrap stdin handle");

      stdin_handle
        .write_all(self.stdin.as_bytes())
        .expect("failed to write stdin to just process");
    }

    let output = child
      .wait_with_output()
      .expect("failed to wait for just process");

    let have = Output {
      status: output.status.code().unwrap(),
      stdout: str::from_utf8(&output.stdout).unwrap(),
      stderr: str::from_utf8(&output.stderr).unwrap(),
    };

    let want = Output {
      status: self.status,
      stdout: &stdout,
      stderr: &stderr,
    };

    pretty_assertions::assert_eq!(have, want, "bad output");

    if self.status == EXIT_SUCCESS {
      test_round_trip(self.directory.path());
    }
  }
}

#[derive(PartialEq, Debug)]
struct Output<'a> {
  stdout: &'a str,
  stderr: &'a str,
  status: i32,
}

fn test_round_trip(tmpdir: &Path) {
  println!("Reparsing...");

  let output = Command::new(&executable_path("just"))
    .current_dir(tmpdir)
    .arg("--dump")
    .output()
    .expect("just invocation failed");

  if !output.status.success() {
    panic!("dump failed: {}", output.status);
  }

  let dumped = String::from_utf8(output.stdout).unwrap();

  let reparsed_path = tmpdir.join("reparsed.just");

  fs::write(&reparsed_path, &dumped).unwrap();

  let output = Command::new(&executable_path("just"))
    .current_dir(tmpdir)
    .arg("--justfile")
    .arg(&reparsed_path)
    .arg("--dump")
    .output()
    .expect("just invocation failed");

  if !output.status.success() {
    panic!("reparse failed: {}", output.status);
  }

  let reparsed = String::from_utf8(output.stdout).unwrap();

  assert_eq!(reparsed, dumped, "reparse mismatch");
}
