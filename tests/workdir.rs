use super::*;

#[test]
fn workdir() {
  Test::new()
    .justfile(
      r#"
      set workdir := 'bar'

      print1:
        echo "$(basename "$PWD")"

      [no-cd]
      print2:
        echo "$(basename "$PWD")"
    "#,
    )
    .current_dir("foo")
    .tree(tree! {
      foo: {},
      bar: {}
    })
    .args(["print1", "print2"])
    .stderr(
      r#"echo "$(basename "$PWD")"
echo "$(basename "$PWD")"
"#,
    )
    .stdout("bar\nfoo\n")
    .run();
}
