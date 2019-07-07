mod tempdir;

#[cfg(unix)]
mod unix {
  use super::tempdir::tempdir;
  use executable_path::executable_path;
  use std::{
    fs,
    process::Command,
    time::{Duration, Instant},
  };

  fn kill(process_id: u32) {
    unsafe {
      libc::kill(process_id as i32, libc::SIGINT);
    }
  }

  fn interrupt_test(justfile: &str) {
    let tmp = tempdir();
    let mut justfile_path = tmp.path().to_path_buf();
    justfile_path.push("justfile");
    fs::write(justfile_path, justfile).unwrap();

    let start = Instant::now();

    let mut child = Command::new(&executable_path("just"))
      .current_dir(&tmp)
      .spawn()
      .expect("just invocation failed");

    while start.elapsed() < Duration::from_millis(500) {}

    kill(child.id());

    let status = child.wait().unwrap();

    let elapsed = start.elapsed();

    if elapsed > Duration::from_secs(2) {
      panic!("process returned too late: {:?}", elapsed);
    }

    if elapsed < Duration::from_millis(100) {
      panic!("process returned too early : {:?}", elapsed);
    }

    assert_eq!(status.code(), Some(130));
  }

  #[test]
  #[ignore]
  fn interrupt_shebang() {
    interrupt_test(
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
      "
foo = `sleep 1`

default:
  @echo {{foo}}
",
    );
  }
}
