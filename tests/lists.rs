use super::*;

#[test]
fn lists_setting_is_unstable() {
  Test::new()
    .justfile("set lists")
    .stderr_regex("error: the `lists` setting is currently unstable.*")
    .failure();
}

#[test]
fn quote_quotes_each_element_of_variadic_arguments() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args:
          @echo "{{ quote(args) }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "baz bob"])
    .stdout("'bar' 'baz bob'\n")
    .success();
}

#[test]
fn quote_of_empty_list_is_empty() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args:
          @echo "bar{{ quote(args) }}baz"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("barbaz\n")
    .success();
}

#[test]
fn quote_of_empty_variadic_is_empty_string_without_lists_setting() {
  Test::new()
    .justfile(
      r#"
        foo *args:
          @echo "bar{{ quote(args) }}baz"
      "#,
    )
    .arg("foo")
    .stdout("bar''baz\n")
    .success();
}

#[test]
fn quote_quotes_single_element_values_whole() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo bar='baz bob':
          @echo "{{ quote(bar) }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("'baz bob'\n")
    .success();
}

#[test]
fn absolute_path_resolves_each_element_of_a_list() {
  let test = Test::new()
    .justfile(
      r#"
        set lists

        foo *args:
          @echo "{{ absolute_path(args) }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "baz bob"]);

  let mut tempdir = test.tempdir.path().to_owned();

  if cfg!(unix) {
    tempdir = tempdir.canonicalize().unwrap();
  }

  test
    .stdout(format!(
      "{} {}\n",
      tempdir.join("bar").to_str().unwrap(),
      tempdir.join("baz bob").to_str().unwrap(),
    ))
    .success();
}

#[test]
fn absolute_path_of_empty_list_is_empty() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args:
          @echo "bar{{ absolute_path(args) }}baz"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("barbaz\n")
    .success();
}

#[test]
fn append_appends_to_each_element_of_a_list() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args:
          @echo "{{ append('.c', args) }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "baz bob"])
    .stdout("bar.c baz bob.c\n")
    .success();
}

#[test]
fn prepend_prepends_to_each_element_of_a_list() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args:
          @echo "{{ prepend('src/', args) }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "baz bob"])
    .stdout("src/bar src/baz bob\n")
    .success();
}

#[test]
fn append_errors_if_first_argument_has_multiple_elements() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args:
          @echo "{{ append(args, 'bar') }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "baz"])
    .stderr(
      r#"
        error: call to function `append` failed: expected `suffix` to be a single element, but it has 2 elements
         ——▶ justfile:4:13
          │
        4 │   @echo "{{ append(args, 'bar') }}"
          │             ^^^^^^
      "#,
    )
    .failure();
}

#[test]
fn append_errors_if_first_argument_is_empty() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args:
          @echo "{{ append(args, 'bar') }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      r#"
        error: call to function `append` failed: expected `suffix` to be a single element, but it has 0 elements
         ——▶ justfile:4:13
          │
        4 │   @echo "{{ append(args, 'bar') }}"
          │             ^^^^^^
      "#,
    )
    .failure();
}

#[test]
fn append_does_not_split_single_strings_with_lists_setting() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo:
          @echo "{{ append('.c', 'foo bar') }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("foo bar.c\n")
    .success();
}

#[test]
fn interpolations_join_lists_with_spaces() {
  Test::new()
    .justfile(
      "
        set lists

        foo *args:
          @echo {{ args }}
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "baz"])
    .stdout("bar baz\n")
    .success();
}

#[test]
fn dependency_arguments_join_lists_with_spaces() {
  Test::new()
    .justfile(
      "
        set lists

        foo *args: (bar args)

        bar first *rest:
          @echo first={{ first }} rest={{ rest }}
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "baz"])
    .stdout("first=bar baz rest=\n")
    .success();
}
