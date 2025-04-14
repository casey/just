use super::*;

#[test]
fn one_successful_recovery() {
  Test::new()
    .justfile(
      "
    foo: || bar
      echo foo
      exit 1

    bar:
      echo bar
  ",
    )
    .stdout(
      "
    foo
    bar
  ",
    )
    .stderr(
      "
    echo foo
    exit 1
    echo bar
  ",
    )
    .run();
}

#[test]
fn two_successful_recoveries() {
  Test::new()
    .justfile(
      "
    foo: || bar bar2
      echo foo
      exit 1

    bar:
      echo bar

    bar2:
      echo bar2
  ",
    )
    .stdout(
      "
    foo
    bar
    bar2
  ",
    )
    .stderr(
      "
    echo foo
    exit 1
    echo bar
    echo bar2
  ",
    )
    .run();
}

#[test]
fn one_failed_recovery() {
  Test::new()
    .justfile(
      "
    foo: || bar
      echo foo
      exit 2

    bar:
      echo bar
      exit 1
  ",
    )
    .stdout(
      "
    foo
    bar
  ",
    )
    .stderr(
      "
    echo foo
    exit 2
    echo bar
    exit 1
    error: Recipe `bar` failed on line 7 with exit code 1
  ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn first_of_two_recoveries_fail() {
  Test::new()
    .justfile(
      "
    foo: || bar bar2
      echo foo
      exit 2

    bar:
      echo bar
      exit 1

    bar2:
      echo bar2
  ",
    )
    .stdout(
      "
    foo
    bar
  ",
    )
    .stderr(
      "
    echo foo
    exit 2
    echo bar
    exit 1
    error: Recipe `bar` failed on line 7 with exit code 1
  ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn recoveries_with_other_dependencies() {
  Test::new()
    .justfile(
      "
    a: b && c || d
      echo a
      exit 1

    b:
      echo b

    c:
      echo c

    d:
      echo d

  ",
    )
    .stdout(
      "
    b
    a
    d
  ",
    )
    .stderr(
      "
    echo b
    echo a
    exit 1
    echo d
  ",
    )
    .run();
}

#[test]
fn circular_dependency() {
  Test::new()
    .justfile(
      "
    foo: || foo
  ",
    )
    .stderr(
      "
    error: Recipe `foo` depends on itself
     ——▶ justfile:1:9
      │
    1 │ foo: || foo
      │         ^^^
  ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn unknown() {
  Test::new()
    .justfile(
      "
    foo: || bar
  ",
    )
    .stderr(
      "
    error: Recipe `foo` has unknown dependency `bar`
     ——▶ justfile:1:9
      │
    1 │ foo: || bar
      │         ^^^
  ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn unknown_argument() {
  Test::new()
    .justfile(
      "
    bar x:

    foo: || (bar y)
  ",
    )
    .stderr(
      "
    error: Variable `y` not defined
     ——▶ justfile:3:14
      │
    3 │ foo: || (bar y)
      │              ^
  ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn argument() {
  Test::new()
    .justfile(
      "
    foo: || (bar 'hello')
      exit 1

    bar x:
      echo {{ x }}
  ",
    )
    .stdout(
      "
    hello
  ",
    )
    .stderr(
      "
    exit 1
    echo hello
  ",
    )
    .run();
}

#[test]
fn duplicate_recoveries_dont_run() {
  Test::new()
    .justfile(
      "
    a: || b c
      echo a
      exit 1

    b: d
      echo b

    c: d
      echo c

    d:
      echo d
  ",
    )
    .stdout(
      "
    a
    d
    b
    c
  ",
    )
    .stderr(
      "
    echo a
    exit 1
    echo d
    echo b
    echo c
  ",
    )
    .run();
}

#[test]
fn recoveries_run_even_if_already_ran_as_prior() {
  Test::new()
    .justfile(
      "
    a: b || b
      echo a
      exit 1

    b:
      echo b
  ",
    )
    .stdout(
      "
    b
    a
    b
  ",
    )
    .stderr(
      "
    echo b
    echo a
    exit 1
    echo b
  ",
    )
    .run();
}
