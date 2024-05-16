#![cfg(target_os = "linux")]
use super::*;

#[test]
fn test_global_justfile() {
  let tmp = temptree! {
    just: {
      justfile: "default:\n   echo 'foo'",

    }
  };

  let xdg_config_path = tmp.path();

  let output = Command::new(executable_path("just"))
    .env("XDG_CONFIG_HOME", xdg_config_path.display().to_string())
    .args(["--global"])
    .output()
    .expect("just invocation failed");

  let expected_status = 0;
  let expected_stdout = "foo\n";
  let expected_stderr = "echo 'foo'\n";

  let mut failure = false;

  let status = output.status.code().unwrap();
  if status != expected_status {
    println!("bad status: {status} != {expected_status}");
    failure = true;
  }

  let stdout = str::from_utf8(&output.stdout).unwrap();
  if stdout != expected_stdout {
    println!("Bad stdout:\ngot:\n{stdout:?}\n\nexpected:\n{expected_stdout:?}");
    failure = true;
  }

  let stderr = str::from_utf8(&output.stderr).unwrap();
  if stderr != expected_stderr {
    println!("Bad stderr:\ngot:\n{stderr:?}\n\nexpected:\n{expected_stderr:?}");
    failure = true;
  }

  if failure {
    panic!("test failed");
  }
}
