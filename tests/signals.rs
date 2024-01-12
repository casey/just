use {
  super::*,
  libc::SIGINT,
  std::time::{Duration, Instant},
};

fn send_signal(process_id: u32, signal: i32) {
  eprintln!("Sending signal {signal:?} to process {process_id:?}");
  #[cfg(unix)]
  unsafe {
    libc::kill(process_id as i32, signal);
  }
  #[cfg(windows)]
  unsafe {
    let res = windows::Win32::System::Console::GenerateConsoleCtrlEvent(
      signal as u32,
      process_id as u32,
    );
    println!("res: {:#?}", res)
  }
}

fn signal_test(arguments: &[&str], justfile: &str, times: u64) {
  log::error!("signal_test");
  let tmp = tempdir();
  let mut justfile_path = tmp.path().to_path_buf();
  justfile_path.push("justfile");
  fs::write(justfile_path, unindent(justfile)).unwrap();

  #[cfg(unix)]
  let signals = [SIGINT, libc::SIGHUP];

  #[cfg(windows)]
  let signals = [SIGINT];

  for signal in signals {
    let start = Instant::now();
    let mut child = Command::new(executable_path("just"))
      .current_dir(&tmp)
      .args(arguments)
      // .stderr(Stdio::null())
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
#[ignore]
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
#[ignore]
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
#[ignore]
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
#[ignore]
fn signal_command() {
  signal_test(&["--command", "sleep", "1"], "", 1);
}

#[test]
#[ignore]
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

          trap 'handle_signal 130' SIGINT

          sleep 1
      ",
    5,
  );
}

#[cfg(unix)]
#[test]
#[ignore]
fn multiple_signals_shebang_unix() {
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

          # In unix we can trap SIGHUP
          trap 'handle_signal 129' SIGHUP
          trap 'handle_signal 130' SIGINT

          sleep 1
      ",
    5,
  );
}
