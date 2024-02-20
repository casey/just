use {
  super::*,
  std::time::{Duration, Instant},
};

use rustix::process::{kill_process, Pid, Signal};

fn interrupt_test(arguments: &[&str], justfile: &str) {
  let tmp = tempdir();
  let mut justfile_path = tmp.path().to_path_buf();
  justfile_path.push("justfile");
  fs::write(justfile_path, unindent(justfile)).unwrap();

  let start = Instant::now();

  let mut child = Command::new(executable_path("just"))
    .current_dir(&tmp)
    .args(arguments)
    .spawn()
    .expect("just invocation failed");

  while start.elapsed() < Duration::from_millis(500) {}

  // FIXME: the old libc implementation ignored errors, this does the same but it's probably not the best idea
  let _ = kill_process(Pid::from_child(&child), Signal::Int);

  let status = child.wait().unwrap();

  let elapsed = start.elapsed();

  if elapsed > Duration::from_secs(2) {
    panic!("process returned too late: {elapsed:?}");
  }

  if elapsed < Duration::from_millis(100) {
    panic!("process returned too early : {elapsed:?}");
  }

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
