use super::*;

#[test]
#[ignore]
fn prior_dependencies_run_in_parallel() {
  let start = Instant::now();

  Test::new()
    .justfile(
      "
        [parallel]
        foo: a b c d e

        a:
          sleep 1

        b:
          sleep 1

        c:
          sleep 1

        d:
          sleep 1

        e:
          sleep 1
      ",
    )
    .stderr(
      "
        sleep 1
        sleep 1
        sleep 1
        sleep 1
        sleep 1
      ",
    )
    .success();

  assert!(start.elapsed() < Duration::from_secs(2));
}

#[test]
#[ignore]
fn subsequent_dependencies_run_in_parallel() {
  let start = Instant::now();

  Test::new()
    .justfile(
      "
        [parallel]
        foo: && a b c d e

        a:
          sleep 1

        b:
          sleep 1

        c:
          sleep 1

        d:
          sleep 1

        e:
          sleep 1
      ",
    )
    .stderr(
      "
        sleep 1
        sleep 1
        sleep 1
        sleep 1
        sleep 1
      ",
    )
    .success();

  assert!(start.elapsed() < Duration::from_secs(2));
}

#[test]
fn parallel_dependencies_report_errors() {
  Test::new()
    .justfile(
      "
        [parallel]
        foo: bar

        bar:
          exit 1
      ",
    )
    .stderr(
      "
        exit 1
        error: Recipe `bar` failed on line 5 with exit code 1
      ",
    )
    .failure();
}
