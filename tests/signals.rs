use {
  super::*,
  libc::{SIGHUP, SIGINT},
  std::time::{Duration, Instant},
};

fn send_signal(process_id: u32, signal: i32) {
  unsafe {
    libc::kill(process_id as i32, signal);
  }
}

fn signal_test(arguments: &[&str], justfile: &str, times: u64) {
  let tmp = tempdir();
  let mut justfile_path = tmp.path().to_path_buf();
  justfile_path.push("justfile");
  fs::write(justfile_path, unindent(justfile)).unwrap();

  let signals = [SIGINT, SIGHUP];
  for signal in signals {
    let start = Instant::now();
    let mut child = Command::new(executable_path("just"))
      .current_dir(&tmp)
      .args(arguments)
      .spawn()
      .expect("just invocation failed");

    let initial_wait = 500;
    let cycle_wait = 50;
    while start.elapsed() < Duration::from_millis(initial_wait) {}

    for i in 0..times {
      // wait a little bit each time we send a signal
      while start.elapsed() < Duration::from_millis(initial_wait + cycle_wait * (i + 1)) {}
      send_signal(child.id(), signal);
    }

    let status = child.wait().expect("failed to wait on child");

    let elapsed = start.elapsed();

    if elapsed > Duration::from_millis(1000) {
      panic!("process returned too late: {elapsed:?}");
    }

    if elapsed < Duration::from_millis(initial_wait + cycle_wait * times) {
      panic!("process returned too early : {elapsed:?}");
    }

    assert_eq!(status.code(), Some(signal + 128));
  }
}

#[test]
fn signal_shebang() {
  signal_test(
    &[],
    "
        default:
          #!/usr/bin/env sh

          sleep 1
      ",
    1,
  );
}

#[test]
fn signal_line() {
  signal_test(
    &[],
    "
        default:
          @sleep 1
      ",
    1,
  );
}

#[test]
fn signal_backtick() {
  signal_test(
    &[],
    "
        foo := `sleep 1`

        default:
          @echo {{foo}}
      ",
    1,
  );
}

#[test]
fn signal_command() {
  signal_test(&["--command", "sleep", "1"], "", 1);
}

#[test]
fn multiple_signals_shebang() {
  signal_test(
    &[],
    "
        default:
          #!/usr/bin/env sh

          counter=0
          handle_signal() {
            ((counter++))
            if ((counter > 5)); then
              exit $1
            fi
          }

          trap 'handle_signal 129' SIGHUP
          trap 'handle_signal 130' SIGINT

          sleep 1
      ",
    5,
  );
}
