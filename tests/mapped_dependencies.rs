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
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "baz", "bob bib"])
    .stdout("bar: baz\nbar: bob bib\n")
    .success();
}

#[test]
fn unstarred_arguments_are_passed_whole_to_each_invocation() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: *(bar args *args)

        bar all arg:
          @echo "all: {{ quote(all) }} arg: {{ arg }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "baz", "bob"])
    .stdout("all: 'baz' 'bob' arg: baz\nall: 'baz' 'bob' arg: bob\n")
    .success();
}

#[test]
fn starred_argument_may_bind_to_variadic_parameter() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: *(bar *args)

        bar *rest:
          @echo "bar: {{ rest }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "baz", "bob"])
    .stdout("bar: baz\nbar: bob\n")
    .success();
}

#[test]
fn empty_list_runs_dependency_zero_times() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: *(bar *args)
          @echo "foo"

        bar arg:
          @echo "bar: {{ arg }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("foo\n")
    .success();
}

#[test]
fn duplicate_elements_are_deduplicated() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: *(bar *args)

        bar arg:
          @echo "bar: {{ arg }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "baz", "baz"])
    .stdout("bar: baz\n")
    .success();
}

#[test]
fn mapped_dependency_works_as_subsequent() {
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
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "baz", "bob"])
    .stdout("foo\nbar: baz\nbar: bob\n")
    .success();
}

#[test]
fn argument_count_is_checked_statically() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: *(bar *args 'baz')

        bar arg:
          @echo "bar: {{ arg }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: dependency `bar` got 2 arguments but takes 1 argument
         ——▶ justfile:3:14
          │
        3 │ foo *args: *(bar *args 'baz')
          │              ^^^
      ",
    )
    .failure();
}

#[test]
fn starred_argument_without_starred_dependency_is_an_error() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: (bar *args)

        bar arg:
          @echo "bar: {{ arg }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: dependency arguments are passed whole; to invoke a dependency once per element of \
        a starred argument, star the dependency, as in `*(recipe *argument)`
         ——▶ justfile:3:17
          │
        3 │ foo *args: (bar *args)
          │                 ^
      ",
    )
    .failure();
}

#[test]
fn starred_dependency_without_starred_argument_is_an_error() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: *(bar args)

        bar arg:
          @echo "bar: {{ arg }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: mapped dependency must star the argument to map over, as in `*(recipe *argument)`
         ——▶ justfile:3:12
          │
        3 │ foo *args: *(bar args)
          │            ^
      ",
    )
    .failure();
}

#[test]
fn multiple_starred_arguments_are_an_error() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: *(bar *args *args)

        bar baz bob:
          @echo "bar"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: mapped dependencies may star only one argument
         ——▶ justfile:3:24
          │
        3 │ foo *args: *(bar *args *args)
          │                        ^
      ",
    )
    .failure();
}

#[test]
fn mapped_dependencies_require_lists_setting() {
  Test::new()
    .justfile(
      r#"
        foo *args: *(bar *args)

        bar arg:
          @echo "bar: {{ arg }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: mapped dependencies require the `lists` setting
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
    .env("JUST_UNSTABLE", "1")
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
