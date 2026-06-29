use super::*;

#[test]
fn mapped_dependency_runs_once_per_element() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: *(bar *args)

        bar arg:
          @echo "bar: {{ arg }}"
      "#,
    )
    .unstable()
    .args(["foo", "baz", "bob bib"])
    .stdout(
      "
        bar: baz
        bar: bob bib
      ",
    )
    .success();
}

#[test]
fn subsequents_may_be_mapped() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: && *(bar *args)
          @echo "foo"

        bar arg:
          @echo "bar: {{ arg }}"
      "#,
    )
    .unstable()
    .args(["foo", "baz", "bob"])
    .stdout(
      "
        foo
        bar: baz
        bar: bob
      ",
    )
    .success();
}

#[test]
fn mapped_dependencies_may_take_unstarred_arguments() {
  Test::new()
    .justfile(
      "
        set lists

        foo *args: *(bar args *args)

        bar all arg:
          @echo 'all: {{ show(all) }} arg: {{ show(arg )}}'
      ",
    )
    .unstable()
    .args(["foo", "baz", "bob"])
    .stdout(
      r#"
        all: ["baz", "bob"] arg: "baz"
        all: ["baz", "bob"] arg: "bob"
      "#,
    )
    .success();
}

#[test]
fn starred_argument_outside_mapped_dependency_error() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: (bar *args)

        bar arg:
          @echo "bar: {{ arg }}"
      "#,
    )
    .unstable()
    .arg("foo")
    .stderr(
      "
        error: starred arguments may not be used outside mapped dependencies
         ——▶ justfile:3:17
          │
        3 │ foo *args: (bar *args)
          │                 ^
      ",
    )
    .failure();
}

#[test]
fn mapped_dependency_without_starred_argument_error() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: *(bar args)

        bar arg:
          @echo "bar: {{ arg }}"
      "#,
    )
    .unstable()
    .arg("foo")
    .stderr(
      "
        error: mapped dependencies must have starred argument
         ——▶ justfile:3:12
          │
        3 │ foo *args: *(bar args)
          │            ^
      ",
    )
    .failure();
}

#[test]
fn multiple_starred_argument_error() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: *(bar *args *args)

        bar baz bob:
          @echo "bar"
      "#,
    )
    .unstable()
    .arg("foo")
    .stderr(
      "
        error: mapped dependencies may not have multiple starred arguments
         ——▶ justfile:3:24
          │
        3 │ foo *args: *(bar *args *args)
          │                        ^
      ",
    )
    .failure();
}

#[test]
fn starred_arguments_require_value() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: *(bar *args + 'bob')

        bar arg:
          @echo "bar: {{ arg }}"
      "#,
    )
    .unstable()
    .arg("foo")
    .stderr(
      "
        error: expected '*', backtick, '!', '[', identifier, '(', ')', '/', or string, but found '+'
         ——▶ justfile:3:24
          │
        3 │ foo *args: *(bar *args + 'bob')
          │                        ^
      ",
    )
    .failure();
}

#[test]
fn starred_argument_may_be_value() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: *(bar *(args + ' bob'))

        bar arg:
          @echo "bar: {{ arg }}"
      "#,
    )
    .unstable()
    .args(["foo", "baz"])
    .stdout(
      "
        bar: baz bob
      ",
    )
    .success();
}

#[test]
fn mapped_dependencies_require_lists() {
  Test::new()
    .justfile(
      r#"
        foo *args: *(bar *args)

        bar arg:
          @echo "bar: {{ arg }}"
      "#,
    )
    .unstable()
    .arg("foo")
    .stderr(
      "
        error: mapped dependencies require `set lists`
         ——▶ justfile:1:14
          │
        1 │ foo *args: *(bar *args)
          │              ^^^
      ",
    )
    .failure();
}

#[test]
fn mapped_dependencies_round_trip_through_dump() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: *(bar *args)

        bar arg:
          @echo "bar: {{ arg }}"
      "#,
    )
    .unstable()
    .arg("--dump")
    .stdout(
      r#"
        set lists

        foo *args: *(bar *args)

        bar arg:
            @echo "bar: {{ arg }}"
      "#,
    )
    .success();
}
