use super::*;

#[test]
fn lists_setting_is_unstable() {
  Test::new()
    .justfile("set lists")
    .stderr_regex("error: the `lists` setting is currently unstable.*")
    .failure();
}

#[test]
fn quote_quotes_each_element_of_a_list() {
  assert_list_eq("quote(['bar', 'baz bob'])", r#"["'bar'", "'baz bob'"]"#);
}

#[test]
fn quote_of_empty_list_is_empty() {
  assert_list_eq("quote([])", "[]");
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
  assert_list_eq("quote('baz bob')", r#""'baz bob'""#);
}

#[test]
fn absolute_path_resolves_each_element_of_a_list() {
  let test = Test::new()
    .justfile("set lists\n\nx := show(absolute_path(['bar', 'baz bob']))")
    .env("JUST_UNSTABLE", "1")
    .args(["--evaluate", "x"]);

  let mut tempdir = test.tempdir.path().to_owned();

  if cfg!(unix) {
    tempdir = tempdir.canonicalize().unwrap();
  }

  test
    .stdout(format!(
      r#"["{}", "{}"]"#,
      tempdir.join("bar").to_str().unwrap().replace('\\', "\\\\"),
      tempdir
        .join("baz bob")
        .to_str()
        .unwrap()
        .replace('\\', "\\\\"),
    ))
    .unindent_stdout(false)
    .success();
}

#[test]
fn absolute_path_of_empty_list_is_empty() {
  assert_list_eq("absolute_path([])", "[]");
}

#[test]
fn append_appends_to_each_element_of_a_list() {
  assert_list_eq(
    "append('.c', ['bar', 'baz bob'])",
    r#"["bar.c", "baz bob.c"]"#,
  );
}

#[test]
fn prepend_prepends_to_each_element_of_a_list() {
  assert_list_eq(
    "prepend('src/', ['bar', 'baz bob'])",
    r#"["src/bar", "src/baz bob"]"#,
  );
}

#[test]
fn append_does_not_split_single_strings_with_lists_setting() {
  assert_list_eq("append('.c', 'foo bar')", r#""foo bar.c""#);
}

#[test]
fn recipe_interpolations_space_join_lists() {
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
fn fstring_interpolations_space_join_lists() {
  assert_list_eq(r#"f"{{['bar', 'baz']}}""#, r#""bar baz""#);
}

#[test]
fn join_list_joins_lists_with_spaces() {
  assert_list_eq("join_list(['bar', 'baz'])", r#""bar baz""#);
}

#[test]
fn join_list_joins_with_separator() {
  assert_list_eq("join_list(['bar', 'baz'], ', ')", r#""bar, baz""#);
}

#[test]
fn join_list_requires_lists_setting() {
  Test::new()
    .justfile(r#"x := join_list("foo")"#)
    .args(["--evaluate", "x"])
    .stderr(
      r#"
        error: the `join_list()` function requires `set lists`
         ——▶ justfile:1:6
          │
        1 │ x := join_list("foo")
          │      ^^^^^^^^^
      "#,
    )
    .failure();
}

#[test]
fn join_list_separator_must_not_be_a_list() {
  Test::new()
    .justfile(
      "
        set lists

        foo:
          @echo {{ join_list(['bar', 'baz'], [',', ';']) }}
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stderr(
      r#"
        error: list value [",", ";"] passed to `join_list()`
        the behavior of lists with many built-in functions is undecided
        see https://github.com/casey/just#lists
         ——▶ justfile:4:12
          │
        4 │   @echo {{ join_list(['bar', 'baz'], [',', ';']) }}
          │            ^^^^^^^^^
      "#,
    )
    .failure();
}

#[test]
fn split_splits_string_on_separator() {
  assert_list_eq("split('foo,bar,baz', ',')", r#"["foo", "bar", "baz"]"#);
}

#[test]
fn split_of_string_not_containing_separator_is_single_element() {
  assert_list_eq("split('foo', ',')", r#""foo""#);
}

#[test]
fn split_keeps_empty_elements_with_explicit_separator() {
  assert_list_eq("split('foo,,bar,', ',')", r#"["foo", "", "bar", ""]"#);
}

#[test]
fn split_without_separator_splits_on_whitespace() {
  assert_list_eq("split('  foo \t bar  baz ')", r#"["foo", "bar", "baz"]"#);
}

#[test]
fn split_without_separator_of_whitespace_is_empty() {
  assert_list_eq("split('  \t ')", "[]");
}

#[test]
fn split_requires_lists_setting() {
  Test::new()
    .justfile(r#"x := split("foo", ",")"#)
    .args(["--evaluate", "x"])
    .stderr(
      r#"
        error: the `split()` function requires `set lists`
         ——▶ justfile:1:6
          │
        1 │ x := split("foo", ",")
          │      ^^^^^
      "#,
    )
    .failure();
}

#[test]
fn split_argument_must_not_be_a_list() {
  Test::new()
    .justfile(
      "
        set lists

        foo:
          @echo {{ split(['bar', 'baz'], ',') }}
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stderr(
      r#"
        error: list value ["bar", "baz"] passed to `split()`
        the behavior of lists with many built-in functions is undecided
        see https://github.com/casey/just#lists
         ——▶ justfile:4:12
          │
        4 │   @echo {{ split(['bar', 'baz'], ',') }}
          │            ^^^^^
      "#,
    )
    .failure();
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
      "
        set lists

        foo *args: (bar args)

        bar *rest:
          @echo '{{ show(rest) }}'
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "baz bob"])
    .stdout(
      r#"
        ["bar", "baz bob"]
      "#,
    )
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
      "
        set lists

        foo *args: (bar args)

        bar first:
          @echo '{{ show(first) }}'
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "baz"])
    .stdout(
      r#"
        ["bar", "baz"]
      "#,
    )
    .success();
}

#[test]
fn dependency_arguments_bind_to_one_parameter_each() {
  Test::new()
    .justfile(
      "
        set lists

        foo *args: (bar 'baz' args)

        bar first *rest:
          @echo '{{ show(first) }} {{ show(rest) }}'
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar", "bob"])
    .stdout(
      r#"
        "baz" ["bar", "bob"]
      "#,
    )
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
    .stderr("error: recipe `bar` parameter `rest` requires at least one element but received empty list\n")
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
    .stderr("error: recipe `bar` parameter `first` requires at least one element but received empty list\n")
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
      "
        set lists

        foo: (bar)

        bar *rest:
          @echo '{{ show(rest) }}'
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("[]\n")
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
      "set lists\nbaz *rest:\n @echo '{{ show(rest) }}'",
    )
    .justfile(
      "
        mod foo

        bar *args: (foo::baz args)
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["bar", "baz", "bob"])
    .stdout(
      r#"
        "baz bob"
      "#,
    )
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

#[test]
fn concatenation_broadcasts_string_over_list() {
  assert_list_eq("'foo' + ['bar', 'baz']", r#"["foobar", "foobaz"]"#);
  assert_list_eq("['bar', 'baz'] + 'foo'", r#"["barfoo", "bazfoo"]"#);
}

#[test]
fn concatenation_combines_equal_length_lists_pairwise() {
  assert_list_eq("['a', 'b'] + ['c', 'd']", r#"["ac", "bd"]"#);
}

#[test]
fn concatenation_of_strings_is_a_string() {
  assert_list_eq("'foo' + 'bar'", r#""foobar""#);
}

#[test]
fn concatenation_of_empty_lists_is_empty() {
  assert_list_eq("[] + []", "[]");
}

#[test]
fn list_concatenation_appends_lists() {
  assert_list_eq("['foo', 'bar'] ++ ['baz']", r#"["foo", "bar", "baz"]"#);
}

#[test]
fn list_concatenation_appends_empty_lists() {
  assert_list_eq("[] ++ ['foo']", r#""foo""#);
  assert_list_eq("['foo'] ++ []", r#""foo""#);
  assert_list_eq("[] ++ []", "[]");
}

#[test]
fn list_concatenation_treats_strings_as_one_element_lists() {
  assert_list_eq("'foo' ++ ['bar']", r#"["foo", "bar"]"#);
  assert_list_eq("['foo'] ++ 'bar'", r#"["foo", "bar"]"#);
}

#[test]
fn list_concatenation_requires_lists_setting() {
  Test::new()
    .justfile("x := 'foo' ++ 'bar'")
    .args(["--evaluate", "x"])
    .stderr(
      r"
        error: operator `++` requires `set lists`
         ——▶ justfile:1:12
          │
        1 │ x := 'foo' ++ 'bar'
          │            ^^
      ",
    )
    .failure();
}

#[test]
fn join_broadcasts_string_over_list() {
  assert_list_eq("'foo' / ['bar', 'baz']", r#"["foo/bar", "foo/baz"]"#);
  assert_list_eq("['bar', 'baz'] / 'foo'", r#"["bar/foo", "baz/foo"]"#);
}

#[test]
fn join_combines_equal_length_lists_pairwise() {
  assert_list_eq("['a', 'b'] / ['c', 'd']", r#"["a/c", "b/d"]"#);
}

#[test]
fn unary_join_broadcasts_over_list() {
  assert_list_eq("/ ['bar', 'baz']", r#"["/bar", "/baz"]"#);
}

#[test]
fn concatenation_with_empty_list_is_an_error() {
  Test::new()
    .justfile("set lists\n\nx := 'foo' + []")
    .env("JUST_UNSTABLE", "1")
    .args(["--evaluate", "x"])
    .stderr(
      r"
        error: operator `+` cannot be applied to empty lists
         ——▶ justfile:3:12
          │
        3 │ x := 'foo' + []
          │            ^
      ",
    )
    .failure();
}

#[test]
fn concatenation_of_different_length_lists_is_an_error() {
  Test::new()
    .justfile("set lists\n\nx := ['a', 'b'] + ['c', 'd', 'e']")
    .env("JUST_UNSTABLE", "1")
    .args(["--evaluate", "x"])
    .stderr(
      r#"
        error: operator `+` cannot be applied to lists of different lengths: ["a", "b"] + ["c", "d", "e"]
         ——▶ justfile:3:17
          │
        3 │ x := ['a', 'b'] + ['c', 'd', 'e']
          │                 ^
      "#,
    )
    .failure();
}

#[test]
fn unary_join_with_empty_list_is_an_error() {
  Test::new()
    .justfile("set lists\n\nx := / []")
    .env("JUST_UNSTABLE", "1")
    .args(["--evaluate", "x"])
    .stderr(
      r"
        error: operator `/` cannot be applied to empty lists
         ——▶ justfile:3:6
          │
        3 │ x := / []
          │      ^
      ",
    )
    .failure();
}

#[test]
fn assert_message_space_joins_lists() {
  Test::new()
    .justfile(
      "
        set lists

        foo:
          {{ assert('a' != 'a', ['foo', 'bar']) }}
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stderr(
      "
        error: assert failed: foo bar
         ——▶ justfile:4:6
          │
        4 │   {{ assert('a' != 'a', ['foo', 'bar']) }}
          │      ^^^^^^
      ",
    )
    .failure();
}

#[test]
fn confirm_prompt_space_joins_lists() {
  Test::new()
    .justfile(
      "
        set lists

        [confirm(['foo', 'bar'])]
        @foo:
          echo FOO
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stderr("foo bar ")
    .stdout("FOO\n")
    .stdin("y")
    .success();
}

#[test]
fn env_attribute_value_space_joins_lists() {
  Test::new()
    .justfile(
      "
        set lists

        [env('FOO', ['bar', 'baz'])]
        foo:
          @echo $FOO
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("bar baz\n")
    .success();
}

#[test]
fn env_attribute_empty_list_leaves_variable_unset() {
  Test::new()
    .justfile(
      r#"
        set lists

        [env('FOO', [])]
        foo:
          @echo "${FOO-unset}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("unset\n")
    .success();
}

#[test]
fn env_attribute_empty_string_sets_variable() {
  Test::new()
    .justfile(
      r#"
        set lists

        [env('FOO', [''])]
        foo:
          @echo "[${FOO-unset}]"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("[]\n")
    .success();
}

#[test]
fn empty_list_export_leaves_variable_unset() {
  Test::new()
    .justfile(
      r#"
        set lists

        export FOO := []

        foo:
          @echo "${FOO-unset}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("unset\n")
    .success();
}

#[test]
fn non_empty_list_export_sets_variable() {
  Test::new()
    .justfile(
      r#"
        set lists

        export FOO := ['bar', 'baz']

        foo:
          @echo "$FOO"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("bar baz\n")
    .success();
}

#[test]
fn empty_list_exported_parameter_leaves_variable_unset() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo $bar=[]:
          @echo "${bar-unset}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("unset\n")
    .success();
}

#[test]
fn empty_list_set_export_leaves_variable_unset() {
  Test::new()
    .justfile(
      r#"
        set lists
        set export

        FOO := []

        foo:
          @echo "${FOO-unset}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("unset\n")
    .success();
}

#[test]
fn list_in_env_attribute_name_points_at_attribute_name() {
  Test::new()
    .justfile(
      "
        set lists

        [env(['FOO', 'BAR'], 'baz')]
        foo:
          @echo hi
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stderr(
      r#"
        error: list value ["FOO", "BAR"] used as `env` attribute name
         ——▶ justfile:3:2
          │
        3 │ [env(['FOO', 'BAR'], 'baz')]
          │  ^^^
      "#,
    )
    .failure();
}

#[test]
fn env_returns_first_present_variable() {
  Test::new()
    .justfile("set lists\n\nx := env(['ZADDY', 'BAR'])")
    .env("JUST_UNSTABLE", "1")
    .env("BAR", "bar")
    .args(["--evaluate", "x"])
    .stdout("bar")
    .unindent_stdout(false)
    .success();
}

#[test]
fn env_stops_at_first_present_variable_including_empty() {
  Test::new()
    .justfile("set lists\n\nx := show(env(['ZADDY', 'BAR']))")
    .env("JUST_UNSTABLE", "1")
    .env("ZADDY", "")
    .env("BAR", "bar")
    .args(["--evaluate", "x"])
    .stdout(r#""""#)
    .unindent_stdout(false)
    .success();
}

#[test]
fn env_returns_default_when_no_variable_present() {
  assert_list_eq("env(['ZADDY', 'XYZ'], 'baz')", r#""baz""#);
}

#[test]
fn env_returns_list_default() {
  assert_list_eq("env(['ZADDY'], ['a', 'b'])", r#"["a", "b"]"#);
}

#[test]
fn env_with_empty_key_list_uses_default() {
  assert_list_eq("env([], 'baz')", r#""baz""#);
}

#[test]
fn env_with_empty_key_list_and_no_default_is_an_error() {
  Test::new()
    .arg("a")
    .justfile(
      "
        set lists

        a:
          echo {{env([])}}
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stderr(
      "
        error: call to function `env` failed: empty environment variable list with no default
         ——▶ justfile:4:10
          │
        4 │   echo {{env([])}}
          │          ^^^
      ",
    )
    .failure();
}

#[test]
fn env_missing_keys_error_names_all_keys() {
  Test::new()
    .arg("a")
    .justfile(
      "
        set lists

        a:
          echo {{env(['ZADDY', 'XYZ'])}}
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stderr(
      "
        error: call to function `env` failed: environment variables `ZADDY` and `XYZ` not present
         ——▶ justfile:4:10
          │
        4 │   echo {{env(['ZADDY', 'XYZ'])}}
          │          ^^^
      ",
    )
    .failure();
}

#[test]
fn env_single_missing_key_keeps_singular_message() {
  Test::new()
    .arg("a")
    .justfile(
      "
        set lists

        a:
          echo {{env('ZADDY')}}
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stderr(
      "
        error: call to function `env` failed: environment variable `ZADDY` not present
         ——▶ justfile:4:10
          │
        4 │   echo {{env('ZADDY')}}
          │          ^^^
      ",
    )
    .failure();
}

#[test]
fn env_var_accepts_list_of_keys() {
  Test::new()
    .justfile("set lists\n\nx := env_var(['ZADDY', 'BAR'])")
    .env("JUST_UNSTABLE", "1")
    .env("BAR", "bar")
    .args(["--evaluate", "x"])
    .stdout("bar")
    .unindent_stdout(false)
    .success();
}

#[test]
fn env_var_or_default_accepts_list_of_keys() {
  assert_list_eq("env_var_or_default(['ZADDY', 'XYZ'], [])", "[]");
}

#[test]
fn list_in_working_directory_attribute_points_at_attribute_name() {
  Test::new()
    .justfile(
      "
        set lists

        [working-directory(['foo', 'bar'])]
        baz:
          @echo hi
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stderr(
      r#"
        error: list value ["foo", "bar"] used as a `[working-directory]` attribute
         ——▶ justfile:3:2
          │
        3 │ [working-directory(['foo', 'bar'])]
          │  ^^^^^^^^^^^^^^^^^
      "#,
    )
    .failure();
}

#[test]
fn interpreter_settings_flatten_lists() {
  Test::new()
    .justfile(
      "
        set lists
        set shell := ['echo', ['foo', 'bar']]

        baz:
          hello
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("foo bar hello\n")
    .stderr("hello\n")
    .shell(false)
    .success();
}

#[test]
fn empty_interpreter_setting_is_an_error() {
  Test::new()
    .justfile(
      "
        set lists
        set shell := [[]]

        foo:
          @echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stderr(
      "
        error: `shell` setting requires at least one element but evaluated to empty list
         ——▶ justfile:2:5
          │
        2 │ set shell := [[]]
          │     ^^^^^
      ",
    )
    .failure();
}

#[test]
fn list_in_function_argument_points_at_function_name() {
  Test::new()
    .justfile(
      "
        set lists

        foo:
          @echo {{ uppercase(['bar', 'baz']) }}
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stderr(
      r#"
        error: list value ["bar", "baz"] passed to `uppercase()`
        the behavior of lists with many built-in functions is undecided
        see https://github.com/casey/just#lists
         ——▶ justfile:4:12
          │
        4 │   @echo {{ uppercase(['bar', 'baz']) }}
          │            ^^^^^^^^^
      "#,
    )
    .failure();
}

#[test]
fn list_in_setting_value_points_at_setting_name() {
  Test::new()
    .justfile(
      "
        set lists
        set tempdir := ['foo', 'bar']

        foo:
          @echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stderr(
      r#"
        error: list value ["foo", "bar"] assigned to `tempdir` setting
         ——▶ justfile:2:5
          │
        2 │ set tempdir := ['foo', 'bar']
          │     ^^^^^^^
      "#,
    )
    .failure();
}
