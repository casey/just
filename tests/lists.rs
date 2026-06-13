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
fn prepend_errors_if_suffix_is_not_single_element() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args:
          @echo "{{ prepend(args, 'bar') }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "baz"])
    .stderr(
      r#"
        error: call to function `prepend` failed: `prefix` must be single element list but has 2 elements
         ——▶ justfile:4:13
          │
        4 │   @echo "{{ prepend(args, 'bar') }}"
          │             ^^^^^^^
      "#,
    )
    .failure();
}

#[test]
fn append_errors_if_suffix_is_not_single_element() {
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
        error: call to function `append` failed: `suffix` must be single element list but has 2 elements
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
fn dependency_arguments_join_lists_without_lists_setting() {
  Test::new()
    .justfile(
      "
        foo *args: (bar args)

        bar first *rest:
          @echo first={{ first }} rest={{ rest }}
      ",
    )
    .args(["foo", "bar", "baz"])
    .stdout("first=bar baz rest=\n")
    .success();
}

#[test]
fn dependency_arguments_forward_lists() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: (bar args)

        bar *rest:
          @echo "{{ quote(rest) }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "baz bob"])
    .stdout("'bar' 'baz bob'\n")
    .success();
}

#[test]
fn dependency_arguments_forward_lists_to_positional_arguments() {
  Test::new()
    .justfile(
      r#"
        set lists
        set positional-arguments

        foo *args: (bar args)

        bar *rest:
          @echo "$1-$2"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "baz"])
    .stdout("bar-baz\n")
    .success();
}

#[test]
fn singular_parameters_contribute_one_positional_argument() {
  Test::new()
    .justfile(
      r#"
        set lists
        set positional-arguments

        foo *args: (bar args 'bob')

        bar first second:
          @echo "$1-$2"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "baz"])
    .stdout("bar baz-bob\n")
    .success();
}

#[test]
fn lists_bind_to_singular_parameters() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: (bar args)

        bar first:
          @echo "{{ quote(first) }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "baz"])
    .stdout("'bar' 'baz'\n")
    .success();
}

#[test]
fn dependency_arguments_bind_to_one_parameter_each() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo *args: (bar 'baz' args)

        bar first *rest:
          @echo "{{ first }} {{ quote(rest) }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "bob"])
    .stdout("baz 'bar' 'bob'\n")
    .success();
}

#[test]
fn variadic_parameters_accept_at_most_one_dependency_argument() {
  Test::new()
    .justfile(
      "
        set lists

        foo *args:

        bar: (foo 'baz' 'bob')
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("bar")
    .stderr(
      "
        error: dependency `foo` got 2 arguments but takes at most 1 argument
         ——▶ justfile:5:7
          │
        5 │ bar: (foo 'baz' 'bob')
          │       ^^^
      ",
    )
    .failure();
}

#[test]
fn empty_list_for_plus_variadic_is_an_error() {
  Test::new()
    .justfile(
      "
        set lists

        foo *args: (bar args)

        bar +rest:
          @echo {{ rest }}
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr("error: recipe `bar` parameter `rest` requires at least one element but received an empty list\n")
    .failure();
}

#[test]
fn empty_list_for_required_parameter_is_an_error() {
  Test::new()
    .justfile(
      "
        set lists

        foo *args: (bar args)

        bar first:
          @echo {{ first }}
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr("error: recipe `bar` parameter `first` requires at least one element but received an empty list\n")
    .failure();
}

#[test]
fn empty_list_for_defaulted_parameter_uses_default() {
  Test::new()
    .justfile(
      "
        set lists

        foo *args: (bar args)

        bar first='baz':
          @echo {{ first }}
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("baz\n")
    .success();
}

#[test]
fn omitted_star_variadic_dependency_argument_is_empty_list() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo: (bar)

        bar *rest:
          @echo "baz{{ quote(rest) }}baz"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("bazbaz\n")
    .success();
}

#[test]
fn lists_forwarded_to_module_without_lists_setting_are_joined() {
  Test::new()
    .write(
      "foo.just",
      "baz first *rest:\n @echo first={{ first }} rest={{ rest }}",
    )
    .justfile(
      "
        set lists

        mod foo

        bar *args: (foo::baz args)
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["bar", "baz", "bob"])
    .stdout("first=baz bob rest=\n")
    .success();
}

#[test]
fn joined_arguments_forwarded_to_module_with_lists_setting_are_single_elements() {
  Test::new()
    .write(
      "foo.just",
      "set lists\nbaz *rest:\n @echo \"{{ quote(rest) }}\"",
    )
    .justfile(
      "
        mod foo

        bar *args: (foo::baz args)
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["bar", "baz", "bob"])
    .stdout("'baz bob'\n")
    .success();
}

#[test]
fn evaluate_prints_lists() {
  Test::new()
    .justfile(
      "
        set lists

        a := 'foo'
        b := ['bar']
        c := ['baz', 'bob']
        d := []
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("--evaluate")
    .stdout(
      r#"
        a := "foo"
        b := "bar"
        c := ["baz", "bob"]
        d := []
      "#,
    )
    .success();
}
