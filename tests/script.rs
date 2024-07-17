use super::*;

#[test]
fn unstable() {
  Test::new()
    .justfile(
      "
        [script('sh', '-u')]
        foo:
          echo FOO

      ",
    )
    .stderr_regex(r"error: The `\[script\]` attribute is currently unstable\..*")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn basic() {
  Test::new()
    .justfile(
      "
        set unstable

        [script('sh', '-u')]
        foo:
          echo FOO

      ",
    )
    .stdout("FOO\n")
    .run();
}

#[test]
fn requires_argument() {
  Test::new()
    .justfile(
      "
        set unstable

        [script]
        foo:
      ",
    )
    .stderr(
      "
        error: Attribute `script` got 0 arguments but takes at least 1 argument
         ——▶ justfile:3:2
          │
        3 │ [script]
          │  ^^^^^^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn not_allowed_with_shebang() {
  Test::new()
    .justfile(
      "
        set unstable

        [script('sh', '-u')]
        foo:
          #!/bin/sh

      ",
    )
    .stderr(
      "
        error: Recipe `foo` has both shebang line and `[script]` attribute
         ——▶ justfile:4:1
          │
        4 │ foo:
          │ ^^^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}
