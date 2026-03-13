use {super::*, nix::sys::signal::Signal, nix::unistd::Pid, std::process::Child};

fn kill(child: &Child, signal: Signal) {
  nix::sys::signal::kill(Pid::from_raw(child.id().try_into().unwrap()), signal).unwrap();
}

fn interrupt_test(arguments: &[&str], justfile: &str) {
  let tmp = tempdir();
  let mut justfile_path = tmp.path().to_path_buf();
  justfile_path.push("justfile");
  fs::write(justfile_path, unindent(justfile)).unwrap();

  let start = Instant::now();

  let mut child = Command::new(JUST)
    .current_dir(&tmp)
    .args(arguments)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("just invocation failed");

  while start.elapsed() < Duration::from_millis(500) {}

  kill(&child, Signal::SIGINT);

  let status = child.wait().unwrap();

  let elapsed = start.elapsed();

  assert!(
    elapsed <= Duration::from_secs(2),
    "process returned too late: {elapsed:?}"
  );

  assert!(
    elapsed >= Duration::from_millis(100),
    "process returned too early : {elapsed:?}"
  );

  assert_eq!(status.code(), Some(130));
}

#[test]
#[ignore]
fn interrupt_shebang() {
  interrupt_test(
    &[],
    "
        default:
          #!/usr/bin/env sh
          sleep 1
      ",
  );
}

#[test]
#[ignore]
fn interrupt_line() {
  interrupt_test(
    &[],
    "
        default:
          @sleep 1
      ",
  );
}

#[test]
#[ignore]
fn interrupt_backtick() {
  interrupt_test(
    &[],
    "
        foo := `sleep 1`

        default:
          @echo {{foo}}
      ",
  );
}

#[test]
#[ignore]
fn interrupt_command() {
  interrupt_test(&["--command", "sleep", "1"], "");
}

// This test is ignored because it is sensitive to the process signal mask.
// Programs like `watchexec` and `cargo-watch` change the signal mask to ignore
// `SIGHUP`, which causes this test to fail.
#[test]
#[ignore]
fn forwarding() {
  let tempdir = tempdir();

  fs::write(
    tempdir.path().join("justfile"),
    "foo:\n @{{just_executable()}} --request '\"signal\"'",
  )
  .unwrap();

  for signal in [Signal::SIGINT, Signal::SIGQUIT, Signal::SIGHUP] {
    let mut child = Command::new(JUST)
      .current_dir(&tempdir)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();

    // wait for child to start
    thread::sleep(Duration::from_millis(500));

    // send non-forwarded signal
    kill(&child, signal);

    // wait for child to receive signal
    thread::sleep(Duration::from_millis(500));

    // assert that child does not exit, because signal is not forwarded
    assert!(child.try_wait().unwrap().is_none());

    // send forwarded signal
    kill(&child, Signal::SIGTERM);

    // child exits
    let output = child.wait_with_output().unwrap();

    let status = output.status;
    let stderr = str::from_utf8(&output.stderr).unwrap();
    let stdout = str::from_utf8(&output.stdout).unwrap();

    let mut failures = 0;

    if status.code() != Some(128 + signal as i32) {
      failures += 1;
      eprintln!("unexpected status: {status}");
    }

    // just reports that it was interrupted by first, non-forwarded signal
    if stderr != format!("error: Interrupted by {signal}\n") {
      failures += 1;
      eprintln!("unexpected stderr: {stderr}");
    }

    // child reports that it was terminated by forwarded signal
    if stdout != r#"{"signal":"SIGTERM"}"# {
      failures += 1;
      eprintln!("unexpected stdout: {stdout}");
    }

    assert!(failures == 0, "{failures} failures");
  }
}

#[test]
#[ignore]
#[cfg(any(
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "ios",
  target_os = "macos",
  target_os = "netbsd",
  target_os = "openbsd",
))]
fn siginfo_prints_current_process() {
  let tempdir = tempdir();

  fs::write(tempdir.path().join("justfile"), "foo:\n @sleep 1").unwrap();

  let child = Command::new(JUST)
    .current_dir(&tempdir)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();

  thread::sleep(Duration::from_millis(500));

  kill(&child, Signal::SIGINFO);

  let output = child.wait_with_output().unwrap();

  let status = output.status;
  let stderr = str::from_utf8(&output.stderr).unwrap();
  let stdout = str::from_utf8(&output.stdout).unwrap();

  let mut failures = 0;

  if !status.success() {
    failures += 1;
    eprintln!("unexpected status: {status}");
  }

  let re =
    Regex::new(r#"just \d+: 1 child process:\n\d+: cd ".*" && "sh" "-cu" "sleep 1"\n"#).unwrap();

  if !re.is_match(stderr) {
    failures += 1;
    eprintln!("unexpected stderr: {stderr}");
  }

  if !stdout.is_empty() {
    failures += 1;
    eprintln!("unexpected stdout: {stdout}");
  }

  assert!(failures == 0, "{failures} failures");
}
