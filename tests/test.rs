use crate::common::*;

pub(crate) use pretty_assertions::assert_eq;

macro_rules! test {
  (
    name:     $name:ident,
    justfile: $justfile:expr,
    $(args:     ($($arg:tt)*),)?
    $(env:      {
      $($env_key:literal : $env_value:literal,)*
    },)?
    $(stdin:    $stdin:expr,)?
    $(stdout:   $stdout:expr,)?
    $(stderr:   $stderr:expr,)?
    $(status:   $status:expr,)?
    $(shell:    $shell:expr,)?
    $(dotenv_load: $dotenv_load:expr,)?
  ) => {
    #[test]
    fn $name() {
      #[allow(unused_mut)]
      let mut env = std::collections::BTreeMap::new();

      $($(env.insert($env_key.to_string(), $env_value.to_string());)*)?

      crate::test::Test {
        justfile: $justfile,
        $(args: &[$($arg)*],)?
        $(stdin: $stdin,)?
        $(stdout: $stdout,)?
        $(stderr: $stderr,)?
        $(status: $status,)?
        $(shell: $shell,)?
        $(dotenv_load: $dotenv_load,)?
        env,
        ..crate::test::Test::default()
      }.run();
    }
  }
}

pub(crate) struct Test<'a> {
  pub(crate) justfile:    &'a str,
  pub(crate) args:        &'a [&'a str],
  pub(crate) env:         BTreeMap<String, String>,
  pub(crate) stdin:       &'a str,
  pub(crate) stdout:      &'a str,
  pub(crate) stderr:      &'a str,
  pub(crate) status:      i32,
  pub(crate) shell:       bool,
  pub(crate) dotenv_load: bool,
}

impl<'a> Default for Test<'a> {
  fn default() -> Test<'a> {
    Test {
      justfile:    "",
      args:        &[],
      env:         BTreeMap::new(),
      stdin:       "",
      stdout:      "",
      stderr:      "",
      status:      EXIT_SUCCESS,
      shell:       true,
      dotenv_load: true,
    }
  }
}

impl<'a> Test<'a> {
  pub(crate) fn run(self) {
    let tmp = tempdir();

    let mut justfile = unindent(self.justfile);

    if self.dotenv_load {
      justfile.push_str("\nset dotenv-load := true\n");
    }

    let stdout = unindent(self.stdout);
    let stderr = unindent(self.stderr);

    let mut justfile_path = tmp.path().to_path_buf();
    justfile_path.push("justfile");
    fs::write(&justfile_path, justfile).unwrap();

    let mut dotenv_path = tmp.path().to_path_buf();
    dotenv_path.push(".env");
    fs::write(dotenv_path, "DOTENV_KEY=dotenv-value").unwrap();

    let mut command = Command::new(&executable_path("just"));

    if self.shell {
      command.args(&["--shell", "bash"]);
    }

    let mut child = command
      .args(self.args)
      .envs(self.env)
      .current_dir(tmp.path())
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

    let mut have = Output {
      status: output.status.code().unwrap(),
      stdout: str::from_utf8(&output.stdout).unwrap(),
      stderr: str::from_utf8(&output.stderr).unwrap(),
    };

    let mut want = Output {
      status: self.status,
      stdout: &stdout,
      stderr: &stderr,
    };

    if let Some(first) = self.args.first() {
      if *first == "--dump" {
        have.stdout = have
          .stdout
          .trim_end_matches("set dotenv-load := true\n")
          .trim_end();
        want.stdout = want.stdout.trim_end();
      }
    }

    assert_eq!(have, want, "bad output");

    if self.status == EXIT_SUCCESS {
      test_round_trip(tmp.path());
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
