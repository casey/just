use super::*;

#[test]
fn linewise() {
  Test::new()
    .justfile(
      "
        [no-cd]
        foo:
          cat bar
      ",
    )
    .current_dir("foo")
    .tree(tree! {
      foo: {
        bar: "hello",
      }
    })
    .stderr("cat bar\n")
    .stdout("hello")
    .success();
}

#[test]
fn shebang() {
  Test::new()
    .justfile(
      "
        [no-cd]
        foo:
          #!/bin/sh
          cat bar
      ",
    )
    .current_dir("foo")
    .tree(tree! {
      foo: {
        bar: "hello",
      }
    })
    .stdout("hello")
    .success();
}

#[test]
fn setting_applies_to_recipes() {
  Test::new()
    .justfile(
      "
        set no-cd := true

        foo:
          cat bar
      ",
    )
    .current_dir("child")
    .tree(tree! {
      bar: "root",
      child: {
        bar: "child",
      }
    })
    .stderr("cat bar\n")
    .stdout("child")
    .success();
}

#[test]
fn working_directory_attribute_overrides_setting() {
  Test::new()
    .justfile(
      "
        set no-cd := true

        [working-directory('workspace')]
        foo:
          cat data.txt
      ",
    )
    .write("workspace/data.txt", "WORKSPACE")
    .stderr("cat data.txt\n")
    .stdout("WORKSPACE")
    .success();
}

#[test]
fn paths_stay_module_dir_without_strict() {
  Test::new()
    .justfile(
      "
        set no-cd := true

        file := `cat data.txt`

        @foo:
          echo {{file}}
      ",
    )
    .current_dir("inv")
    .write("data.txt", "MODULE")
    .write("inv/data.txt", "INVOCATION")
    .stdout("MODULE\n")
    .success();
}
