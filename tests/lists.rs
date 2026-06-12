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
      "
        set lists

        foo *args:
          @echo \"{{ quote(args) }}\"
      ",
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
      "
        set lists

        foo *args:
          @echo \"bar{{ quote(args) }}baz\"
      ",
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
      "
        foo *args:
          @echo \"bar{{ quote(args) }}baz\"
      ",
    )
    .arg("foo")
    .stdout("bar''baz\n")
    .success();
}

#[test]
fn quote_quotes_single_element_values_whole() {
  Test::new()
    .justfile(
      "
        set lists

        foo bar='baz bob':
          @echo \"{{ quote(bar) }}\"
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("'baz bob'\n")
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
