use super::*;

#[test]
fn all() {
  Test::new()
    .justfile(
      "
        [dragonfly]
        [freebsd]
        [linux]
        [macos]
        [netbsd]
        [no-exit-message]
        [openbsd]
        [android]
        [unix]
        [windows]
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
      "
        error: recipe attribute `no-exit-message` first used on line 1 is duplicated on line 2
         ——▶ justfile:2:2
          │
        2 │ [no-exit-message]
          │  ^^^^^^^^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn multiple_attributes_one_line() {
  Test::new()
    .justfile(
      "
        [macos,windows,linux,openbsd,freebsd,dragonfly,netbsd,android]
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
        [macos,windows linux,openbsd,freebsd,dragonfly,netbsd,android]
        [no-exit-message]
        foo:
          exit 1
      ",
    )
    .stderr(
      "
        error: expected ']', ':', ',', or '(', but found identifier
         ——▶ justfile:1:16
          │
        1 │ [macos,windows linux,openbsd,freebsd,dragonfly,netbsd,android]
          │                ^^^^^
          ",
    )
    .failure();
}

#[test]
fn multiple_attributes_one_line_duplicate_check() {
  Test::new()
    .justfile(
      "
        [macos, windows, linux, openbsd, freebsd, dragonfly, netbsd, android]
        [linux]
        foo:
          exit 1
      ",
    )
    .stderr(
      "
        error: recipe attribute `linux` first used on line 1 is duplicated on line 2
         ——▶ justfile:2:2
          │
        2 │ [linux]
          │  ^^^^^
      ",
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
      "
        error: attribute `private` got 1 argument but takes 0 arguments
         ——▶ justfile:1:2
          │
        1 │ [private('foo')]
          │  ^^^^^^^
          ",
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
      "
        error: attribute `metadata` got 0 arguments but takes at least 1 argument
         ——▶ justfile:1:2
          │
        1 │ [metadata]
          │  ^^^^^^^^
          ",
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
      "
        error: recipe `baz` has invalid attribute `extension`
         ——▶ justfile:2:1
          │
        2 │ baz:
          │ ^^^
      ",
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
      "
        error: recipe attribute `confirm` first used on line 1 is duplicated on line 2
         ——▶ justfile:2:2
          │
        2 │ [confirm: 'no']
          │  ^^^^^^^
      ",
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
fn env_attribute_with_expression() {
  Test::new()
    .justfile(
      "
        suffix := 'world'

        [env('GREETING', 'hello ' + suffix)]
        foo:
          @echo $GREETING
      ",
    )
    .stdout("hello world\n")
    .success();
}

#[test]
fn env_attribute_name_with_expression() {
  Test::new()
    .justfile(
      "
        prefix := 'MY_'

        [env(prefix + 'VAR', 'value')]
        foo:
          @echo $MY_VAR
      ",
    )
    .stdout("value\n")
    .success();
}

#[test]
fn env_attribute_with_expression_in_script() {
  Test::new()
    .justfile(
      "
        suffix := 'world'

        [env('GREETING', 'hello ' + suffix)]
        foo:
          #!/bin/sh
          echo $GREETING
      ",
    )
    .stdout("hello world\n")
    .success();
}

#[test]
fn env_attribute_in_recipe_params() {
  Test::new()
    .justfile(
      "
        [env('foo', 'bar')]
        baz x=`echo ${foo}.txt`:
            @echo {{x}}
      ",
    )
    .stdout("bar.txt\n")
    .success();
}

#[test]
fn env_attribute_value_cannot_reference_parameter() {
  Test::new()
    .justfile(
      "
        [env('TARGET', target)]
        deploy target:
      ",
    )
    .stderr(
      "
        error: variable `target` not defined
         ——▶ justfile:1:16
          │
        1 │ [env('TARGET', target)]
          │                ^^^^^^
      ",
    )
    .failure();
}

#[test]
fn env_attribute_name_cannot_reference_parameter() {
  Test::new()
    .justfile(
      "
        [env(name, 'value')]
        foo name:
      ",
    )
    .stderr(
      "
        error: variable `name` not defined
         ——▶ justfile:1:6
          │
        1 │ [env(name, 'value')]
          │      ^^^^
      ",
    )
    .failure();
}

#[test]
fn env_attribute_expression_dump() {
  Test::new()
    .justfile(
      "
        suffix := 'world'

        [env('GREETING', 'hello ' + suffix)]
        foo:
          echo $GREETING
      ",
    )
    .arg("--dump")
    .stdout(
      "
        suffix := 'world'

        [env('GREETING', 'hello ' + suffix)]
        foo:
            echo $GREETING
      ",
    )
    .success();
}

#[test]
fn env_attribute_not_in_env_function() {
  Test::new()
    .justfile(
      "

        [env('foo', 'bar')]
        baz:
          @echo {{ env('foo') }}.txt

      ",
    )
    .stderr(
      "
        error: call to function `env` failed: environment variable `foo` not present
         ——▶ justfile:4:12
          │
        4 │   @echo {{ env('foo') }}.txt
          │            ^^^
      ",
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
      "
        error: attribute `env` got 1 argument but takes 2 arguments
         ——▶ justfile:1:2
          │
        1 │ [env('MY_VAR')]
          │  ^^^
      ",
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
      "
        error: attribute `env` got 3 arguments but takes 2 arguments
         ——▶ justfile:1:2
          │
        1 │ [env('A', 'B', 'C')]
          │  ^^^
      ",
    )
    .failure();
}

#[test]
fn env_attribute_overrides_export() {
  Test::new()
    .justfile(
      "
        export FOO := 'export'

        [env('FOO', 'attribute')]
        bar:
          @echo $FOO
      ",
    )
    .stdout("attribute\n")
    .success();
}

#[test]
fn env_attribute_overrides_export_in_script() {
  Test::new()
    .justfile(
      "
        export FOO := 'export'

        [env('FOO', 'attribute')]
        bar:
          #!/bin/sh
          echo $FOO
      ",
    )
    .stdout("attribute\n")
    .success();
}

#[test]
fn env_attribute_duplicate_last_wins() {
  Test::new()
    .justfile(
      "
        [env('VAR1', 'value1')]
        [env('VAR1', 'value 2')]
        foo:
          @echo $VAR1
      ",
    )
    .stdout("value 2\n")
    .success();
}
