use super::*;

#[track_caller]
fn case(justfile: &str, args: &[&str], stdout: &str) {
  Test::new()
    .justfile(justfile)
    .args(args)
    .stdout(stdout)
    .success();
}

#[test]
fn lists_setting_is_unstable() {
  Test::new()
    .justfile("set lists")
    .stderr_regex("error: the `lists` setting is currently unstable.*")
    .failure();
}

#[test]
fn quote_distributes_over_lists() {
  case(
    "
      set lists
      set unstable

      foo *args:
        @printf '%s\\n' {{ quote(args) }}
    ",
    &["foo", "bar", "baz bob"],
    "bar\nbaz bob\n",
  );

  case(
    "
      set lists
      set unstable

      foo *args:
        @printf '%s\\n' bar {{ quote(args) }} baz
    ",
    &["foo"],
    "bar\nbaz\n",
  );

  case(
    "
      foo *args:
        @printf '%s\\n' bar {{ quote(args) }} baz
    ",
    &["foo"],
    "bar\n\nbaz\n",
  );

  case(
    "
      set lists
      set unstable

      foo bar='baz bob':
        @printf '%s\\n' {{ quote(bar) }}
    ",
    &["foo"],
    "baz bob\n",
  );
}

#[test]
fn lists_are_joined_with_spaces_when_consumed_as_strings() {
  case(
    "
      set lists
      set unstable

      foo *args:
        @echo {{ args }}
    ",
    &["foo", "bar", "baz"],
    "bar baz\n",
  );

  case(
    "
      set lists
      set unstable

      foo *args: (bar args)

      bar first *rest:
        @echo first={{ first }} rest={{ rest }}
    ",
    &["foo", "bar", "baz"],
    "first=bar baz rest=\n",
  );
}
