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
        error: recipe `bar` failed on line 5 with exit code 1
      ",
    )
    .failure();
}

#[test]
#[ignore]
fn dependents_block_on_running_dependencies() {
  Test::new()
    .justfile(
      "
        set quiet

        [parallel]
        a: b c
          echo a

        b: x
          echo b

        c: x
          echo c

        x:
          sleep 1
          echo x
      ",
    )
    .stdout_regex(
      r"(?x)
      x\n
      (
        b\nc\n
        |
        c\nb\n
      )
      a\n",
    )
    .success();
}

#[test]
#[ignore]
fn jobs_limits_concurrent_recipes() {
  Test::new()
    .args(["--jobs", "1"])
    .justfile(
      "
        set quiet

        [parallel]
        foo: a b

        a:
          echo a
          sleep 1
          echo a

        b:
          echo b
          sleep 1
          echo b
      ",
    )
    .stdout_regex("(a\na\nb\nb\n|b\nb\na\na\n)")
    .success();
}

#[test]
#[ignore]
fn recipes_up_to_job_limit_run_in_parallel() {
  let start = Instant::now();

  Test::new()
    .args(["--jobs", "2"])
    .justfile(
      "
        [parallel]
        foo: a b

        a:
          sleep 1

        b:
          sleep 1
      ",
    )
    .stderr(
      "
        sleep 1
        sleep 1
      ",
    )
    .success();

  assert!(start.elapsed() < Duration::from_secs(2));
}

#[test]
fn zero_jobs_is_an_error() {
  Test::new()
    .args(["--jobs", "0"])
    .justfile("")
    .stderr(
      "
        error: invalid value '0' for '--jobs <N>': number would be zero for non-zero type

        For more information, try '--help'.
      ",
    )
    .status(2);
}
