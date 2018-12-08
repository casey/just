extern crate brev;
extern crate executable_path;
extern crate libc;
extern crate tempdir;

use executable_path::executable_path;
use std::{
  process::Command,
  thread,
  time::{Duration, Instant},
};
use tempdir::TempDir;

#[cfg(unix)]
fn kill(process_id: u32) {
  unsafe {
    libc::kill(process_id as i32, libc::SIGINT);
  }
}

#[cfg(unix)]
fn interrupt_test(justfile: &str) {
  let tmp = TempDir::new("just-interrupts").unwrap_or_else(|err| {
    panic!(
      "integration test: failed to create temporary directory: {}",
      err
    )
  });

  let mut justfile_path = tmp.path().to_path_buf();
  justfile_path.push("justfile");
  brev::dump(justfile_path, justfile);

  let start = Instant::now();

  let mut child = Command::new(&executable_path("just"))
    .current_dir(&tmp)
    .spawn()
    .expect("just invocation failed");

  thread::sleep(Duration::new(1, 0));

  kill(child.id());

  let status = child.wait().unwrap();

  let elapsed = start.elapsed();

  if elapsed > Duration::new(2, 500_000_000) {
    panic!("process returned too late: {:?}", elapsed);
  }

  if elapsed < Duration::new(1, 500_000_000) {
    panic!("process returned too early : {:?}", elapsed);
  }

  assert_eq!(status.code(), Some(130));
}

#[cfg(unix)]
#[test]
fn interrupt_shebang() {
  interrupt_test(
    "
default:
  #!/usr/bin/env sh
  sleep 2
",
  );
}

#[cfg(unix)]
#[test]
fn interrupt_line() {
  interrupt_test(
    "
default:
  @sleep 2
",
  );
}

#[cfg(unix)]
#[test]
fn interrupt_backtick() {
  interrupt_test(
    "
foo = `sleep 2`

default:
  @echo hello
",
  );
}
