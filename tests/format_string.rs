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
        error: expected backtick, '!', '[', identifier, '(', '/', or string, but found end of file
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
  assert_eval(r#"f"{{{{""#, "{{");
}

#[test]
fn delimiter_may_be_escaped_in_single_quoted_strings() {
  assert_eval("f'{{{{'", "{{");
}

#[test]
fn escaped_delimiter_is_ignored_in_normal_strings() {
  assert_eval("'{{{{'", "{{{{");
}

#[test]
fn escaped_delimiter_in_single_quoted_format_string() {
  assert_eval(r"f'\{{{{'", "\\{{");
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
  assert_eval(r#"f"\u{61}{{"b"}}\u{63}{{"d"}}\u{65}""#, "abcde");
}

#[test]
fn single_quotes_do_not_process_escapes() {
  assert_eval(r"f'\n{{'a'}}\n{{'b'}}\n'", r"\na\nb\n");
}

#[test]
fn indented_format_strings() {
  assert_eval("f'''\n  a\n  {{'b'}}\n  c\n'''", "a\nb\nc\n");
}

#[test]
fn un_indented_format_strings() {
  assert_eval("f'\n  a\n  {{'b'}}\n  c\n'", "\n  a\n  b\n  c\n");
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
        error: variable `bar` not defined
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

#[test]
fn unterminated_format_string_error() {
  Test::new()
    .justfile("x := f'{{}}")
    .stderr(
      "
        error: unterminated string
         ——▶ justfile:1:10
          │
        1 │ x := f'{{}}
          │          ^^
      ",
    )
    .failure();
}

#[test]
fn mismatched_closing_delimiter_in_format_string() {
  Test::new()
    .justfile(
      "
        foo := f'{{ )
        bar:
      ",
    )
    .stderr(
      "
        error: mismatched closing delimiter `)`, did you mean to close the `{{` on line 1?
         ——▶ justfile:1:13
          │
        1 │ foo := f'{{ )
          │             ^
      ",
    )
    .failure();
}

#[test]
fn format_backticks_are_forbidden() {
  Test::new()
    .justfile("foo := f`echo {{ arch() }}`")
    .stderr(
      "
        error: expected '&&', '!=', '!~', '||', comment, end of file, end of line, '==', '=~', '(', '+', '++', or '/', but found backtick
         ——▶ justfile:1:9
          │
        1 │ foo := f`echo {{ arch() }}`
          │         ^^^^^^^^^^^^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn stray_identifier_in_interpolation_is_an_error() {
  Test::new()
    .justfile(
      "
        a := 'A'
        x := f'{{ a x }}plain'
      ",
    )
    .args(["--evaluate", "x"])
    .stderr(
      "
        error: expected '&&', '!=', '!~', '||', '==', '=~', format string, '(', '+', '++', or \
       '/', but found identifier
         ——▶ justfile:2:13
          │
        2 │ x := f'{{ a x }}plain'
          │             ^
      ",
    )
    .failure();
}
