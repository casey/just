use super::*;

test! {
  name: recipe_exit_message_suppressed,
  justfile: r#"
# This is a doc comment
[no-exit-message]
hello:
  @echo "Hello, World!"
  @exit 100
"#,
  stdout:   "Hello, World!\n",
  stderr:   "",
  status:   100,
}

test! {
  name: silent_recipe_exit_message_suppressed,
  justfile: r#"
# This is a doc comment
[no-exit-message]
@hello:
  echo "Hello, World!"
  exit 100
"#,
  stdout:   "Hello, World!\n",
  stderr:   "",
  status:   100,
}

test! {
  name: recipe_has_doc_comment,
  justfile: r"
# This is a doc comment
[no-exit-message]
hello:
  @exit 100
",
  args: ("--list"),
  stdout: "
    Available recipes:
        hello # This is a doc comment
  ",
}

test! {
  name: unknown_attribute,
  justfile: r"
# This is a doc comment
[unknown-attribute]
hello:
  @exit 100
",
  stderr: r"
error: Unknown attribute `unknown-attribute`
 ——▶ justfile:2:2
  │
2 │ [unknown-attribute]
  │  ^^^^^^^^^^^^^^^^^
",
  status: EXIT_FAILURE,
}

test! {
  name: empty_attribute,
  justfile: r"
# This is a doc comment
[]
hello:
  @exit 100
",
  stderr: r"
error: Expected identifier, but found ']'
 ——▶ justfile:2:2
  │
2 │ []
  │  ^
",
  status: EXIT_FAILURE,
}

test! {
  name: extraneous_attribute_before_comment,
  justfile: r"
[no-exit-message]
# This is a doc comment
hello:
  @exit 100
",
  stderr: r"
error: Extraneous attribute
 ——▶ justfile:1:1
  │
1 │ [no-exit-message]
  │ ^
",

  status: EXIT_FAILURE,
}

test! {
  name: extraneous_attribute_before_empty_line,
  justfile: r"
[no-exit-message]

hello:
  @exit 100
",
  stderr: "
    error: Extraneous attribute
     ——▶ justfile:1:1
      │
    1 │ [no-exit-message]
      │ ^
  ",
  status: EXIT_FAILURE,
}

test! {
  name: shebang_exit_message_suppressed,
  justfile: r"
[no-exit-message]
hello:
  #!/usr/bin/env bash
  echo 'Hello, World!'
  exit 100
",
  stdout: "Hello, World!\n",
  stderr: "",
  status: 100,
}

#[test]
fn recipe_exit_message_setting_suppressed() {
  Test::new()
    .justfile(
      r#"
      set no-exit-message

      # This is a doc comment
      hello:
        @echo "Hello, World!"
        @exit 100
    "#,
    )
    .stdout("Hello, World!\n")
    .status(100)
    .run();
}


#[test]
fn shebang_exit_message_setting_suppressed() {
  Test::new()
    .justfile(
      "
      set no-exit-message

      hello:
        #!/usr/bin/env bash
        echo 'Hello, World!'
        exit 100
    ",
    )
    .stdout("Hello, World!\n")
    .status(100)
    .run();
}

#[test]
fn exit_message_override_no_exit_setting() {
  Test::new()
    .justfile(
      "
      set no-exit-message
      
      [exit-message]
      fail:
        @exit 100
    ",
    )
    .status(100)
    .stderr("error: Recipe `fail` failed on line 5 with exit code 100\n")
    .run();
}

#[test]
fn exit_message_override_no_exit_attribute() {
  Test::new()
    .justfile(
      "
      [exit-message, no-exit-message]
      fail:
        @exit 100
    ",
    )
    .status(100)
    .stderr("error: Recipe `fail` failed on line 3 with exit code 100\n")
    .run();
}

#[test]
fn exit_message_override_no_exit_setting_and_attribute() {
  Test::new()
    .justfile(
      "
      set no-exit-message

      [exit-message, no-exit-message]
      fail:
        @exit 100
    ",
    )
    .status(100)
    .stderr("error: Recipe `fail` failed on line 5 with exit code 100\n")
    .run();
}