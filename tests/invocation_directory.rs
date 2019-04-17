use executable_path::executable_path;
use std::{fs, path::Path, process, str};
use tempdir::TempDir;

#[cfg(unix)]
fn to_shell_path(path: &Path) -> String {
  use std::fs;
  fs::canonicalize(path)
    .expect("canonicalize failed")
    .to_str()
    .map(str::to_string)
    .expect("unicode decode failed")
}

#[cfg(windows)]
fn to_shell_path(path: &Path) -> String {
  // Translate path from windows style to unix style
  let mut cygpath = process::Command::new("cygpath");
  cygpath.arg("--unix");
  cygpath.arg(path);
  brev::output(cygpath).expect("converting cygwin path failed")
}

#[test]
fn test_invocation_directory() {
  let tmp = TempDir::new("just-integration").unwrap_or_else(|err| {
    panic!(
      "integration test: failed to create temporary directory: {}",
      err
    )
  });

  let mut justfile_path = tmp.path().to_path_buf();
  justfile_path.push("justfile");
  fs::write(
    justfile_path,
    "default:\n @cd {{invocation_directory()}}\n @echo {{invocation_directory()}}",
  )
  .unwrap();

  let mut subdir = tmp.path().to_path_buf();
  subdir.push("subdir");
  fs::create_dir(&subdir).unwrap();

  let output = process::Command::new(&executable_path("just"))
    .current_dir(&subdir)
    .args(&["--shell", "sh"])
    .output()
    .expect("just invocation failed");

  let mut failure = false;

  let expected_status = 0;
  let expected_stdout = to_shell_path(&subdir) + "\n";
  let expected_stderr = "";

  let status = output.status.code().unwrap();
  if status != expected_status {
    println!("bad status: {} != {}", status, expected_status);
    failure = true;
  }

  let stdout = str::from_utf8(&output.stdout).unwrap();
  if stdout != expected_stdout {
    println!(
      "bad stdout:\ngot:\n{:?}\n\nexpected:\n{:?}",
      stdout, expected_stdout
    );
    failure = true;
  }

  let stderr = str::from_utf8(&output.stderr).unwrap();
  if stderr != expected_stderr {
    println!(
      "bad stderr:\ngot:\n{:?}\n\nexpected:\n{:?}",
      stderr, expected_stderr
    );
    failure = true;
  }

  if failure {
    panic!("test failed");
  }
}
