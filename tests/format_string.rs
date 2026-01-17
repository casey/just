use super::*;

#[test]
fn empty() {
  Test::new()
    .justfile(
      "
        foo := f''

        @baz:
          echo {{foo}}
      ",
    )
    .stdout("\n")
    .unindent_stdout(false)
    .success();
}

#[test]
fn simple() {
  Test::new()
    .justfile(
      "
        foo := f'bar'

        @baz:
          echo {{foo}}
      ",
    )
    .stdout("bar\n")
    .success();
}

#[test]
fn compound() {
  Test::new()
    .justfile(
      "
        bar := 'BAR'
        foo := f'FOO{{ bar }}BAZ'

        @baz:
          echo {{foo}}
      ",
    )
    .stdout("FOOBARBAZ\n")
    .success();
}

#[test]
fn newline() {
  Test::new()
    .justfile(
      "
        bar := 'BAR'
        foo := f'FOO{{
          bar + 'XYZ'
        }}BAZ'

        @baz:
          echo {{foo}}
      ",
    )
    .stdout("FOOBARXYZBAZ\n")
    .success();
}

#[test]
fn conditional() {
  Test::new()
    .justfile(
      "
        foo := f'FOO{{
          if 'a' == 'b' { 'c' } else { 'd' }
        }}BAZ'

        @baz:
          echo {{foo}}
      ",
    )
    .stdout("FOOdBAZ\n")
    .success();
}

#[test]
fn conditional_no_whitespace() {
  Test::new()
    .justfile(
      "
        foo := f'FOO{{if 'a' == 'b' { 'c' } else { 'd' }}}BAZ'

        @baz:
          echo {{foo}}
      ",
    )
    .stdout("FOOdBAZ\n")
    .success();
}

#[test]
fn inner_delimiter() {
  Test::new()
    .justfile(
      "
        bar := 'BAR'
        foo := f'FOO{{(bar)}}BAZ'

        @baz:
          echo {{foo}}
      ",
    )
    .stdout("FOOBARBAZ\n")
    .success();
}

#[test]
fn nested() {
  Test::new()
    .justfile(
      "
        bar := 'BAR'
        foo := f'FOO{{f'[{{bar}}]'}}BAZ'

        @baz:
          echo {{foo}}
      ",
    )
    .stdout("FOO[BAR]BAZ\n")
    .success();
}

#[test]
fn recipe_body() {
  Test::new()
    .justfile(
      "
        bar := 'BAR'
        @baz:
          echo {{f'FOO{{f'[{{bar}}]'}}BAZ'}}
      ",
    )
    .stdout("FOO[BAR]BAZ\n")
    .success();
}

#[test]
fn unclosed() {
  Test::new()
    .justfile("foo := f'FOO{{")
    .stderr(
      "
        error: Expected backtick, identifier, '(', '/', or string, but found end of file
         ——▶ justfile:1:15
          │
        1 │ foo := f'FOO{{
          │               ^
      ",
    )
    .failure();
}

#[test]
fn unmatched_close_is_ignored() {
  Test::new()
    .justfile(
      "
        foo := f'}}'

        @baz:
          echo {{foo}}
      ",
    )
    .stdout("}}\n")
    .unindent_stdout(false)
    .success();
}

#[test]
fn delimiter_may_be_escaped_in_double_quoted_strings() {
  Test::new()
    .justfile(
      r#"
        foo := f"{{{{"
      "#,
    )
    .args(["--evaluate", "foo"])
    .stdout("{{")
    .success();
}

#[test]
fn delimiter_may_be_escaped_in_single_quoted_strings() {
  Test::new()
    .justfile(
      "
        foo := f'{{{{'
      ",
    )
    .args(["--evaluate", "foo"])
    .stdout("{{")
    .success();
}

#[test]
fn escaped_delimiter_is_ignored_in_normal_strings() {
  Test::new()
    .justfile(
      "
        foo := '{{{{'
      ",
    )
    .args(["--evaluate", "foo"])
    .stdout("{{{{")
    .success();
}

#[test]
fn escaped_delimiter_in_single_quoted_format_string() {
  Test::new()
    .justfile(
      r"
        foo := f'\{{{{'
      ",
    )
    .args(["--evaluate", "foo"])
    .stdout("\\{{")
    .success();
}

#[test]
fn escaped_delimiter_in_double_quoted_format_string() {
  Test::new()
    .justfile(
      r#"
        foo := f"\{{{{"
      "#,
    )
    .args(["--evaluate", "foo"])
    .stderr(
      r#"
        error: `\{` is not a valid escape sequence
         ——▶ justfile:1:9
          │
        1 │ foo := f"\{{{{"
          │         ^^^^^^^
      "#,
    )
    .failure();
}

#[test]
fn double_quotes_process_escapes() {
  Test::new()
    .justfile(
      r#"
        foo := f"\u{61}{{"b"}}\u{63}{{"d"}}\u{65}"
      "#,
    )
    .args(["--evaluate", "foo"])
    .stdout("abcde")
    .success();
}

#[test]
fn single_quotes_do_not_process_escapes() {
  Test::new()
    .justfile(
      r#"
        foo := f'\n{{"a"}}\n{{"b"}}\n'
      "#,
    )
    .args(["--evaluate", "foo"])
    .stdout(r"\na\nb\n")
    .success();
}

#[test]
fn indented_format_strings() {
  Test::new()
    .justfile(
      r#"
        foo := f'''
          a
          {{"b"}}
          c
        '''
      "#,
    )
    .args(["--evaluate", "foo"])
    .stdout("a\nb\nc\n")
    .success();
}

#[test]
fn un_indented_format_strings() {
  Test::new()
    .justfile(
      r#"
        foo := f'
          a
          {{"b"}}
          c
        '
      "#,
    )
    .args(["--evaluate", "foo"])
    .stdout("\n  a\n  b\n  c\n")
    .unindent_stdout(false)
    .success();
}

#[test]
fn dump() {
  #[track_caller]
  fn case(string: &str) {
    Test::new()
      .justfile(format!(
        "
          foo := {string}
        "
      ))
      .arg("--dump")
      .stdout(format!("foo := {string}\n"))
      .success();
  }
  case("f''");
  case("f''''''");
  case(r#"f"""#);
  case(r#"f"""""""#);
  case("f'{{'a'}}b{{'c'}}d'");
  case("f'''{{'a'}}b{{'c'}}d'''");
  case(r#"f"""{{'a'}}b{{'c'}}d""""#);
}

#[test]
fn undefined_variable_error() {
  Test::new()
    .justfile(
      "
        foo := f'{{bar}}'
      ",
    )
    .stderr(
      "
        error: Variable `bar` not defined
         ——▶ justfile:1:12
          │
        1 │ foo := f'{{bar}}'
          │            ^^^
      ",
    )
    .failure();
}

#[test]
fn format_string_followed_by_recipe() {
  Test::new()
    .justfile(
      "
        foo := f'{{'foo'}}{{'bar'}}'
        bar:
      ",
    )
    .success();
}
