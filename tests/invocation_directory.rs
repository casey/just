extern crate brev;
extern crate executable_path;
extern crate libc;
extern crate target;
extern crate tempdir;

use executable_path::executable_path;
use std::process;
use std::{str, fs};
use tempdir::TempDir;

#[test]
fn test_invocation_directory() {
  let tmp = TempDir::new("just-integration")
    .unwrap_or_else(
      |err| panic!("integration test: failed to create temporary directory: {}", err));

  let mut justfile_path = tmp.path().to_path_buf();
  justfile_path.push("justfile");
  brev::dump(justfile_path, "default:\n @echo {{invocation_directory()}}");

  let mut dotenv_path = tmp.path().to_path_buf();
  dotenv_path.push(".env");
  brev::dump(dotenv_path, "DOTENV_KEY=dotenv-value");

  let mut subdir = tmp.path().to_path_buf();
  subdir.push("subdir");
  brev::mkdir(&subdir);

  let output = process::Command::new(&executable_path("just"))
    .current_dir(&subdir)
    .args(&["--shell", "sh"])
    .output()
    .expect("just invocation failed");

  let mut failure = false;

  let expected_status = 0;
  let expected_stdout =
    fs::canonicalize(subdir).expect("canonicalize failed")
      .to_str().expect("converting path to unicode failed").to_owned() + "\n";
  let expected_stderr = "";

  let status = output.status.code().unwrap();
  if status != expected_status {
    println!("bad status: {} != {}", status, expected_status);
    failure = true;
  }

  let stdout = str::from_utf8(&output.stdout).unwrap();
  if stdout != expected_stdout {
    println!("bad stdout:\ngot:\n{:?}\n\nexpected:\n{:?}", stdout, expected_stdout);
    failure = true;
  }

  let stderr = str::from_utf8(&output.stderr).unwrap();
  if stderr != expected_stderr {
    println!("bad stderr:\ngot:\n{:?}\n\nexpected:\n{:?}", stderr, expected_stderr);
    failure = true;
  }

  if failure {
    panic!("test failed");
  }
}
