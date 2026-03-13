use super::*;

#[test]
fn summary() {
  Test::new()
    .arg("--summary")
    .justfile(
      "b: a
a:
d: c
c: b
_z: _y
_y:
",
    )
    .stdout("a b c d\n")
    .success();
}

#[test]
fn summary_sorted() {
  Test::new()
    .arg("--summary")
    .justfile(
      "
b:
c:
a:
",
    )
    .stdout("a b c\n")
    .success();
}

#[test]
fn summary_unsorted() {
  Test::new()
    .arg("--summary")
    .arg("--unsorted")
    .justfile(
      "
b:
c:
a:
",
    )
    .stdout("b c a\n")
    .success();
}

#[test]
fn summary_none() {
  Test::new()
    .arg("--summary")
    .arg("--quiet")
    .justfile("")
    .stdout("\n\n\n")
    .success();
}

#[test]
fn no_recipes() {
  Test::new()
    .arg("--summary")
    .stderr("Justfile contains no recipes.\n")
    .stdout("\n\n\n")
    .success();
}

#[test]
fn submodule_recipes() {
  Test::new()
    .write("foo.just", "mod bar\nfoo:")
    .write("bar.just", "mod baz\nbar:")
    .write("baz.just", "mod biz\nbaz:")
    .write("biz.just", "biz:")
    .justfile(
      "
        mod foo

        bar:
      ",
    )
    .arg("--summary")
    .stdout("bar foo::foo foo::bar::bar foo::bar::baz::baz foo::bar::baz::biz::biz\n")
    .success();
}

#[test]
fn summary_implies_unstable() {
  Test::new()
    .write("foo.just", "foo:")
    .justfile(
      "
        mod foo
      ",
    )
    .arg("--summary")
    .stdout("foo::foo\n")
    .success();
}
