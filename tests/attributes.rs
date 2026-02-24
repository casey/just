use super::*;

#[test]
fn all() {
  Test::new()
    .justfile(
      "
      [macos]
      [linux]
      [openbsd]
      [unix]
      [windows]
      [no-exit-message]
      foo:
        exit 1
    ",
    )
    .stderr("exit 1\n")
    .failure();
}

#[test]
fn duplicate_attributes_are_disallowed() {
  Test::new()
    .justfile(
      "
      [no-exit-message]
      [no-exit-message]
      foo:
        echo bar
    ",
    )
    .stderr(
      r#"Error: Duplicate attribute `no-exit-message`
   ╭─[ justfile:2:2 ]
   │
 1 │ [no-exit-message]
   │ ─────────┬────────  
   │          ╰────────── original
 2 │ [no-exit-message]
   │  ───────┬───────  
   │         ╰───────── duplicate
───╯
"#,
    )
    .failure();
}

#[test]
fn multiple_attributes_one_line() {
  Test::new()
    .justfile(
      "
      [macos,windows,linux,openbsd]
      [no-exit-message]
      foo:
        exit 1
    ",
    )
    .stderr("exit 1\n")
    .failure();
}

#[test]
fn multiple_attributes_one_line_error_message() {
  Test::new()
    .justfile(
      "
      [macos,windows linux,openbsd]
      [no-exit-message]
      foo:
        exit 1
    ",
    )
    .stderr(
      r#"Error: Expected ']', ':', ',', or '(', but found identifier
   ╭─[ justfile:1:16 ]
   │
 1 │ [macos,windows linux,openbsd]
───╯
"#,
    )
    .failure();
}

#[test]
fn multiple_attributes_one_line_duplicate_check() {
  Test::new()
    .justfile(
      "
      [macos, windows, linux, openbsd]
      [linux]
      foo:
        exit 1
    ",
    )
    .stderr(
      r#"Error: Duplicate attribute `linux`
   ╭─[ justfile:2:2 ]
   │
 1 │ [macos, windows, linux, openbsd]
   │ ────────────────┬────────────────  
   │                 ╰────────────────── original
 2 │ [linux]
   │  ──┬──  
   │    ╰──── duplicate
───╯
"#,
    )
    .failure();
}

#[test]
fn unexpected_attribute_argument() {
  Test::new()
    .justfile(
      "
      [private('foo')]
      foo:
        exit 1
    ",
    )
    .stderr(
      r#"Error: Attribute argument count mismatch
   ╭─[ justfile:1:2 ]
   │
 1 │ [private('foo')]
   │  ───┬───  
   │     ╰───── Found 1 argument
   │ 
   │ Note: `private` takes 0 arguments
───╯
"#,
    )
    .failure();
}

#[test]
fn multiple_metadata_attributes() {
  Test::new()
    .justfile(
      "
      [metadata('example')]
      [metadata('sample')]
      [no-exit-message]
      foo:
        exit 1
    ",
    )
    .stderr("exit 1\n")
    .failure();
}

#[test]
fn multiple_metadata_attributes_with_multiple_args() {
  Test::new()
    .justfile(
      "
      [metadata('example', 'arg1')]
      [metadata('sample', 'argument')]
      [no-exit-message]
      foo:
        exit 1
    ",
    )
    .stderr("exit 1\n")
    .failure();
}

#[test]
fn expected_metadata_attribute_argument() {
  Test::new()
    .justfile(
      "
      [metadata]
      foo:
        exit 1
    ",
    )
    .stderr(
      r#"Error: Attribute argument count mismatch
   ╭─[ justfile:1:2 ]
   │
 1 │ [metadata]
   │  ────┬───  
   │      ╰───── Found 0 arguments
   │ 
   │ Note: `metadata` takes between 1 and 18446744073709551615 arguments
───╯
"#,
    )
    .failure();
}

#[test]
fn doc_attribute() {
  Test::new()
    .justfile(
      "
    # Non-document comment
    [doc('The real docstring')]
    foo:
      echo foo
  ",
    )
    .args(["--list"])
    .stdout(
      "
    Available recipes:
        foo # The real docstring
        ",
    )
    .success();
}

#[test]
fn doc_attribute_suppress() {
  Test::new()
    .justfile(
      "
        # Non-document comment
        [doc]
        foo:
          echo foo
      ",
    )
    .args(["--list"])
    .stdout(
      "
    Available recipes:
        foo
        ",
    )
    .success();
}

#[test]
fn doc_multiline() {
  Test::new()
    .justfile(
      "
        [doc('multiline
        comment')]
        foo:
      ",
    )
    .args(["--list"])
    .stdout(
      "
    Available recipes:
        # multiline
        # comment
        foo
        ",
    )
    .success();
}

#[test]
fn extension() {
  Test::new()
    .justfile(
      "
        [extension: '.txt']
        baz:
          #!/bin/sh
          echo $0
      ",
    )
    .stdout_regex(r"*baz\.txt\n")
    .success();
}

#[test]
fn extension_on_linewise_error() {
  Test::new()
    .justfile(
      "
        [extension: '.txt']
        baz:
      ",
    )
    .stderr(
      r#"Error: Recipe `baz` has invalid attribute `extension`
   ╭─[ justfile:2:1 ]
   │
 2 │ baz:
───╯
"#,
    )
    .failure();
}

#[test]
fn duplicate_non_repeatable_attributes_are_forbidden() {
  Test::new()
    .justfile(
      "
        [confirm: 'yes']
        [confirm: 'no']
        baz:
      ",
    )
    .stderr(
      r#"Error: Duplicate attribute `confirm`
   ╭─[ justfile:2:2 ]
   │
 1 │ [confirm: 'yes']
   │ ────────┬────────  
   │         ╰────────── original
 2 │ [confirm: 'no']
   │  ───┬───  
   │     ╰───── duplicate
───╯
"#,
    )
    .failure();
}

#[test]
fn shell_expanded_strings_can_be_used_in_attributes() {
  Test::new()
    .justfile(
      "
        [doc(x'foo')]
        bar:
      ",
    )
    .success();
}

#[test]
fn env_attribute_single() {
  Test::new()
    .justfile(
      "
        [env('MY_VAR', 'my_value')]
        foo:
          @echo $MY_VAR
      ",
    )
    .stdout("my_value\n")
    .success();
}

#[test]
fn env_attribute_multiple() {
  Test::new()
    .justfile(
      "
        [env('VAR1', 'value1')]
        [env('VAR2', 'value 2')]
        foo:
          @echo $VAR1 $VAR2
      ",
    )
    .stdout("value1 value 2\n")
    .success();
}

#[test]
fn env_attribute_in_recipe_params() {
  Test::new()
    .justfile(
      r#"
[env("foo", "bar")]
baz x=`echo ${foo}.txt`:
    @echo {{x}}
"#,
    )
    .stdout("bar.txt\n")
    .success();
}

#[test]
fn env_attribute_not_in_env_function() {
  Test::new()
    .justfile(
      r#"

[env("foo", "bar")]
baz:
  @echo {{ env("foo") }}.txt

    "#,
    )
    .stderr(
      r#"
error: Call to function `env` failed: environment variable `foo` not present
 ——▶ justfile:4:12
  │
4 │   @echo {{ env("foo") }}.txt
  │            ^^^

"#,
    )
    .failure();
}

#[test]
fn env_attribute_too_few_arguments() {
  Test::new()
    .justfile(
      "
        [env('MY_VAR')]
        foo:
          echo bar
      ",
    )
    .stderr(
      r#"Error: Attribute argument count mismatch
   ╭─[ justfile:1:2 ]
   │
 1 │ [env('MY_VAR')]
   │  ─┬─  
   │   ╰─── Found 1 argument
   │ 
   │ Note: `env` takes 2 arguments
───╯
"#,
    )
    .failure();
}

#[test]
fn env_attribute_too_many_arguments() {
  Test::new()
    .justfile(
      "
        [env('A', 'B', 'C')]
        foo:
          echo bar
      ",
    )
    .stderr(
      r#"Error: Attribute argument count mismatch
   ╭─[ justfile:1:2 ]
   │
 1 │ [env('A', 'B', 'C')]
   │  ─┬─  
   │   ╰─── Found 3 arguments
   │ 
   │ Note: `env` takes 2 arguments
───╯
"#,
    )
    .failure();
}

#[test]
fn env_attribute_duplicate_error() {
  Test::new()
    .justfile(
      "
        [env('VAR1', 'value1')]
        [env('VAR1', 'value 2')]
        foo:
          @echo $VAR1
      ",
    )
    .stderr(
      r#"Error: Environment variable `VAR1` first set on line 1 is set again on line 2
   ╭─[ justfile:2:2 ]
   │
 2 │ [env('VAR1', 'value 2')]
───╯
"#,
    )
    .failure();
}
