use std::{process::Command, str};

use executable_path::executable_path;

use test_utilities::{assert_stdout, tmptree};

const JUSTFILE_POWERSHELL: &str = r#"
set shell := ["powershell.exe", "-c"]

default:
    #!powershell
    Write-Host Hello-World
"#;

/// Test powershell shebang
#[test]
#[cfg_attr(unix, ignore)]
fn powershell() {
  let tmp = tmptree! {
    justfile: JUSTFILE_POWERSHELL,
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .output()
    .unwrap();

  let stdout = "Hello-World\n";

  assert_stdout(&output, stdout);
}