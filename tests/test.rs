use {
  super::*,
  once_cell::sync::Lazy,
  pretty_assertions::{assert_eq, StrComparison},
};

pub(crate) struct Output {
  pub(crate) pid: u32,
  pub(crate) stdout: String,
  pub(crate) tempdir: TempDir,
}

#[must_use]
pub(crate) struct Test {
  pub(crate) args: Vec<String>,
  pub(crate) current_dir: PathBuf,
  pub(crate) env: BTreeMap<String, String>,
  pub(crate) expected_files: BTreeMap<PathBuf, Vec<u8>>,
  pub(crate) justfile: Option<String>,
  pub(crate) normalize_xtrace: bool,
  pub(crate) response: Option<Response>,
  pub(crate) shell: bool,
  pub(crate) status: i32,
  pub(crate) stderr: String,
  pub(crate) stderr_fn: Option<Box<dyn Fn(&str) -> Result<(), String>>>,
  pub(crate) stderr_regex: Option<Regex>,
  pub(crate) stdin: String,
  pub(crate) stdout: String,
  pub(crate) stdout_regex: Option<Regex>,
  pub(crate) tempdir: TempDir,
  pub(crate) test_round_trip: bool,
  pub(crate) unindent_stdout: bool,
}

impl Test {
  pub(crate) fn new() -> Self {
    Self::with_tempdir(tempdir())
  }

  pub(crate) fn with_tempdir(tempdir: TempDir) -> Self {
    Self {
      args: Vec::new(),
      current_dir: PathBuf::new(),
      env: BTreeMap::new(),
      expected_files: BTreeMap::new(),
      justfile: Some(String::new()),
      response: None,
      shell: true,
      status: EXIT_SUCCESS,
      stderr: String::new(),
      stderr_regex: None,
      stderr_fn: None,
      stdin: String::new(),
      stdout: String::new(),
      stdout_regex: None,
      tempdir,
      test_round_trip: true,
      unindent_stdout: true,
      normalize_xtrace: true,
    }
  }

  pub(crate) fn arg(mut self, val: &str) -> Self {
    self.args.push(val.to_owned());
    self
  }

  pub(crate) fn args<'a>(mut self, args: impl AsRef<[&'a str]>) -> Self {
    for arg in args.as_ref() {
      self = self.arg(arg);
    }
    self
  }

  pub(crate) fn create_dir(self, path: impl AsRef<Path>) -> Self {
    fs::create_dir_all(self.tempdir.path().join(path)).unwrap();
    self
  }

  pub(crate) fn current_dir(mut self, path: impl AsRef<Path>) -> Self {
    path.as_ref().clone_into(&mut self.current_dir);
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
    self.tempdir.path().join("justfile")
  }

  #[cfg(unix)]
  #[track_caller]
  pub(crate) fn symlink(self, original: &str, link: &str) -> Self {
    std::os::unix::fs::symlink(
      self.tempdir.path().join(original),
      self.tempdir.path().join(link),
    )
    .unwrap();
    self
  }

  pub(crate) fn no_justfile(mut self) -> Self {
    self.justfile = None;
    self
  }

  pub(crate) fn response(mut self, response: Response) -> Self {
    self.response = Some(response);
    self.stdout_regex(".*")
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

  pub(crate) fn stderr_regex(mut self, stderr_regex: impl AsRef<str>) -> Self {
    self.stderr_regex = Some(Regex::new(&format!("^(?s){}$", stderr_regex.as_ref())).unwrap());
    self
  }

  pub(crate) fn stderr_fn(mut self, f: impl Fn(&str) -> Result<(), String> + 'static) -> Self {
    self.stderr_fn = Some(Box::from(f));
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

  pub(crate) fn stdout_regex(mut self, stdout_regex: impl AsRef<str>) -> Self {
    self.stdout_regex = Some(Regex::new(&format!("(?s)^{}$", stdout_regex.as_ref())).unwrap());
    self
  }

  pub(crate) fn test_round_trip(mut self, test_round_trip: bool) -> Self {
    self.test_round_trip = test_round_trip;
    self
  }

  pub(crate) fn tree(self, mut tree: Tree) -> Self {
    tree.map(|_name, content| unindent(content));
    tree.instantiate(self.tempdir.path()).unwrap();
    self
  }

  pub(crate) fn unindent_stdout(mut self, unindent_stdout: bool) -> Self {
    self.unindent_stdout = unindent_stdout;
    self
  }

  pub(crate) fn normalize_xtrace(mut self, normalize_xtrace: bool) -> Self {
    self.normalize_xtrace = normalize_xtrace;
    self
  }

  pub(crate) fn write(self, path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> Self {
    let path = self.tempdir.path().join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
    self
  }

  pub(crate) fn make_executable(self, path: impl AsRef<Path>) -> Self {
    let file = self.tempdir.path().join(path);

    // Make sure it exists first, as a sanity check.
    assert!(file.exists(), "file does not exist: {}", file.display());

    // Windows uses file extensions to determine whether a file is executable.
    // Other systems don't care. To keep these tests cross-platform, just make
    // sure all executables end with ".exe" suffix.
    assert!(
      file.extension() == Some("exe".as_ref()),
      "executable file does not end with .exe: {}",
      file.display()
    );

    #[cfg(unix)]
    {
      let perms = std::os::unix::fs::PermissionsExt::from_mode(0o755);
      fs::set_permissions(file, perms).unwrap();
    }

    self
  }

  pub(crate) fn expect_file(mut self, path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> Self {
    let path = path.as_ref();
    self
      .expected_files
      .insert(path.into(), content.as_ref().into());
    self
  }
}

static XTRACE_RE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(
    r#"(?mx)^                  # multiline, verbose
    (?P<pfx>\u{1b}\[[\d;]+m)?  # match the color prefix, if available
    \+.*bash.*?                # match the command. Currently all tests that pass `--verbose` are
                               # run on bash, so this matcher is "good enough" for now.
    (?P<sfx>\u{1b}\[[\d;]+m)?  # match the color suffix, if available
    $"#,
  )
  .unwrap()
});

impl Test {
  #[track_caller]
  pub(crate) fn run(self) -> Output {
    fn compare<T: PartialEq + Debug>(name: &str, have: T, want: T) -> bool {
      let equal = have == want;
      if !equal {
        eprintln!("Bad {name}: {}", Comparison::new(&have, &want));
      }
      equal
    }

    fn compare_string(name: &str, have: &str, want: &str) -> bool {
      let equal = have == want;
      if !equal {
        eprintln!("Bad {name}: {}", StrComparison::new(&have, &want));
      }
      equal
    }

    if let Some(justfile) = &self.justfile {
      let justfile = unindent(justfile);
      fs::write(self.justfile_path(), justfile).unwrap();
    }

    let stdout = if self.unindent_stdout {
      unindent(&self.stdout)
    } else {
      self.stdout.clone()
    };

    let stderr = unindent(&self.stderr);

    let mut command = Command::new(executable_path("just"));

    if self.shell {
      command.args(["--shell", "bash"]);
    }

    let mut child = command
      .args(&self.args)
      .envs(&self.env)
      .current_dir(self.tempdir.path().join(&self.current_dir))
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .expect("just invocation failed");

    let pid = child.id();

    {
      let mut stdin_handle = child.stdin.take().expect("failed to unwrap stdin handle");

      stdin_handle
        .write_all(self.stdin.as_bytes())
        .expect("failed to write stdin to just process");
    }

    let output = child
      .wait_with_output()
      .expect("failed to wait for just process");

    let output_stdout = str::from_utf8(&output.stdout).unwrap();
    let mut output_stderr = str::from_utf8(&output.stderr).unwrap();

    // The xtrace output can differ by working directory, shell, and flags. Normalize it.
    let tmp;
    if self.normalize_xtrace {
      tmp = XTRACE_RE.replace_all(output_stderr, "${pfx}+ COMMAND_XTRACE${sfx}");
      output_stderr = tmp.as_ref();
    }

    if let Some(ref stdout_regex) = self.stdout_regex {
      assert!(
        stdout_regex.is_match(output_stdout),
        "Stdout regex mismatch:\n{output_stdout:?}\n!~=\n/{stdout_regex:?}/",
      );
    }

    if let Some(ref stderr_regex) = self.stderr_regex {
      assert!(
        stderr_regex.is_match(output_stderr),
        "Stderr regex mismatch:\n{output_stderr:?}\n!~=\n/{stderr_regex:?}/",
      );
    }

    if let Some(ref stderr_fn) = self.stderr_fn {
      match stderr_fn(output_stderr) {
        Ok(()) => (),
        Err(e) => panic!("Stderr function mismatch: {e}\n{output_stderr:?}\n",),
      }
    }

    if !compare("status", output.status.code(), Some(self.status))
      | (self.stdout_regex.is_none() && !compare_string("stdout", output_stdout, &stdout))
      | (self.stderr_regex.is_none()
        && self.stderr_fn.is_none()
        && !compare_string("stderr", output_stderr, &stderr))
    {
      panic!("Output mismatch.");
    }

    if let Some(ref response) = self.response {
      assert_eq!(
        &serde_json::from_str::<Response>(output_stdout)
          .expect("failed to deserialize stdout as response"),
        response,
        "response mismatch"
      );
    }

    for (path, expected) in &self.expected_files {
      let actual = fs::read(self.tempdir.path().join(path)).unwrap();
      assert_eq!(
        actual,
        expected.as_slice(),
        "mismatch for expected file at path {}",
        path.display(),
      );
    }

    if self.test_round_trip && self.status == EXIT_SUCCESS {
      self.round_trip();
    }

    Output {
      pid,
      stdout: output_stdout.into(),
      tempdir: self.tempdir,
    }
  }

  fn round_trip(&self) {
    let output = Command::new(executable_path("just"))
      .current_dir(self.tempdir.path())
      .arg("--dump")
      .envs(&self.env)
      .output()
      .expect("just invocation failed");

    assert!(
      output.status.success(),
      "dump failed: {} {:?}",
      output.status,
      output,
    );

    let dumped = String::from_utf8(output.stdout).unwrap();

    let reparsed_path = self.tempdir.path().join("reparsed.just");

    fs::write(&reparsed_path, &dumped).unwrap();

    let output = Command::new(executable_path("just"))
      .current_dir(self.tempdir.path())
      .arg("--justfile")
      .arg(&reparsed_path)
      .arg("--dump")
      .envs(&self.env)
      .output()
      .expect("just invocation failed");

    assert!(output.status.success(), "reparse failed: {}", output.status);

    let reparsed = String::from_utf8(output.stdout).unwrap();

    assert_eq!(reparsed, dumped, "reparse mismatch");
  }
}

pub fn assert_eval_eq(expression: &str, result: &str) {
  Test::new()
    .justfile(format!("x := {expression}"))
    .args(["--evaluate", "x"])
    .stdout(result)
    .unindent_stdout(false)
    .run();
}
