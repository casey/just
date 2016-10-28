extern crate tempdir;
extern crate brev;

use tempdir::TempDir;
use std::process::Command;

fn integration_test(
  name:            &str,
  args:            &[&str],
  justfile:        &str,
  expected_status: i32,
  expected_stdout: &str,
  expected_stderr: &str,
) {
  let tmp = TempDir::new(name)
    .unwrap_or_else(|err| panic!("tmpdir: failed to create temporary directory: {}", err));
  let mut path = tmp.path().to_path_buf();
  path.push("justfile");
  brev::dump(path, justfile);
  let mut binary = std::env::current_dir().unwrap();
  binary.push("./target/debug/j");
  let output = Command::new(binary)
    .current_dir(tmp.path())
    .args(args)
    .output()
    .expect("j invocation failed");

  let mut failure = false;

  let status = output.status.code().unwrap();
  if status != expected_status {
    println!("bad status: {} != {}", status, expected_status);
    failure = true;
  }

  let stdout = std::str::from_utf8(&output.stdout).unwrap();
  if stdout != expected_stdout {
    println!("bad stdout: {:?} != {:?}", stdout, expected_stdout);
    failure = true;
  }

  let stderr = std::str::from_utf8(&output.stderr).unwrap();
  if stderr != expected_stderr {
    println!("bad stderr: {:?} != {:?}", stderr, expected_stderr);
    failure = true;
  }

  if failure {
    panic!("test failed");
  }
}

#[test]
fn simple() {
  integration_test(
    "simple",
    &[],
    "default:\n echo hello",
    0,
    "hello\n",
    "echo hello\n",
  )
}

#[test]
fn quiet() {
  integration_test(
    "quiet",
    &[],
    "default:\n @echo hello",
    0,
    "hello\n",
    "",
  )
}

#[test]
fn order() {
  let text = "
b: a
  echo b
  @mv a b

a:
  echo a
  @touch F
  @touch a

d: c
  echo d
  @rm c

c: b
  echo c
  @mv b c";
  integration_test(
    "order",
    &["a", "d"],
    text,
    0,
    "a\nb\nc\nd\n",
    "echo a\necho b\necho c\necho d\n",
  );
}
