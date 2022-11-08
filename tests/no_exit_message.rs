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
  justfile: r#"
# This is a doc comment
[no-exit-message]
hello:
  @exit 100
"#,
  args: ("--list"),
  stdout: "
    Available recipes:
        hello # This is a doc comment
  ",
}

test! {
  name: unknown_attribute,
  justfile: r#"
# This is a doc comment
[unknown-attribute]
hello:
  @exit 100
"#,
  stderr: r#"
error: Unknown attribute `unknown-attribute`
  |
2 | [unknown-attribute]
  |  ^^^^^^^^^^^^^^^^^
"#,
  status: EXIT_FAILURE,
}

test! {
  name: empty_attribute,
  justfile: r#"
# This is a doc comment
[]
hello:
  @exit 100
"#,
  stderr: r#"
error: Expected identifier, but found ']'
  |
2 | []
  |  ^
"#,
  status: EXIT_FAILURE,
}

test! {
  name: unattached_attribute_before_comment,
  justfile: r#"
[no-exit-message]
# This is a doc comment
hello:
  @exit 100
"#,
  stderr: r#"
error: Expected '@', '[', or identifier, but found comment
  |
2 | # This is a doc comment
  | ^^^^^^^^^^^^^^^^^^^^^^^
"#,

  status: EXIT_FAILURE,
}

test! {
  name: unattached_attribute_before_empty_line,
  justfile: r#"
[no-exit-message]

hello:
  @exit 100
"#,
  stderr: "error: Expected '@', '[', or identifier, but found end of line\n  |\n2 | \n  | ^\n",
  status: EXIT_FAILURE,
}

test! {
  name: shebang_exit_message_suppressed,
  justfile: r#"
[no-exit-message]
hello:
  #!/usr/bin/env bash
  echo 'Hello, World!'
  exit 100
"#,
  stdout: "Hello, World!\n",
  stderr: "",
  status: 100,
}
