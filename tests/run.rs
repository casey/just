use super::*;

#[test]
fn dont_run_duplicate_recipes() {
  Test::new()
    .justfile(
      "
        @foo:
          echo foo
      ",
    )
    .args(["foo", "foo"])
    .stdout("foo\n")
    .success();
}

#[test]
fn one_flag_only_allows_one_invocation() {
  Test::new()
    .justfile(
      "
        @foo:
          echo foo
      ",
    )
    .args(["--one", "foo"])
    .stdout("foo\n")
    .success();

  Test::new()
    .justfile(
      "
        @foo:
          echo foo

        @bar:
          echo bar
      ",
    )
    .args(["--one", "foo", "bar"])
    .stderr("error: expected 1 command-line recipe invocation but found 2\n")
    .failure();
}

#[test]
fn time_reports_time_when_specified() {
  Test::new()
    .justfile(
      "
        foo:
          @echo FOO
      ",
    )
    .arg("--time")
    .stdout("FOO\n")
    .stderr_regex(r"---> foo completed in \d+\.\d+s\n")
    .success();
}

#[test]
fn dependency_traversal_is_not_exponential() {
  let n = 40;

  let mut justfile = String::new();

  for i in 0..n {
    justfile.push_str(&format!("r{i}: r{} r{}\n", i + 1, i + 1));
  }

  justfile.push_str(&format!("r{n}:\n"));

  let tempdir = tempdir();

  fs::write(tempdir.path().join("justfile"), justfile).unwrap();

  let mut child = Command::new(JUST)
    .args(["--dry-run", "r0"])
    .current_dir(tempdir.path())
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .unwrap();

  let start = Instant::now();

  while start.elapsed() < Duration::from_secs(60) {
    if let Some(status) = child.try_wait().unwrap() {
      assert!(status.success());
      return;
    }

    thread::sleep(Duration::from_millis(100));
  }

  child.kill().unwrap();
  child.wait().unwrap();

  panic!("dependency graph traversal of {n} recipes did not complete within 60 seconds");
}
