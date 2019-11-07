use std::{process::Command, str};

use executable_path::executable_path;

use test_utilities::{assert_stdout, tmptree};

const JUSTFILE: &str = "
expression := `EXPRESSION`

recipe default=`DEFAULT`:
  {{expression}}
  {{default}}
  RECIPE
";

/// Test that --shell correctly sets the shell
#[test]
fn shell() {
  let tmp = tmptree! {
    justfile: JUSTFILE,
    shell: "#!/usr/bin/env bash\necho \"$@\"",
  };

  let shell = tmp.path().join("shell");

  #[cfg(unix)]
  {
    let permissions = std::os::unix::fs::PermissionsExt::from_mode(0o700);
    std::fs::set_permissions(&shell, permissions).unwrap();
  }

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--shell")
    .arg(shell)
    .output()
    .unwrap();

  let stdout = "-cu -cu EXPRESSION\n-cu -cu DEFAULT\n-cu RECIPE\n";

  assert_stdout(&output, stdout);
}
