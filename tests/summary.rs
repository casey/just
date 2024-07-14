use super::*;

test! {
  name:     summary,
  justfile: "b: a
a:
d: c
c: b
_z: _y
_y:
",
  args:     ("--summary"),
  stdout:   "a b c d\n",
}

test! {
  name:     summary_sorted,
  justfile: "
b:
c:
a:
",
  args:     ("--summary"),
  stdout:   "a b c\n",
}

test! {
  name:     summary_unsorted,
  justfile: "
b:
c:
a:
",
  args:     ("--summary", "--unsorted"),
  stdout:   "b c a\n",
}

test! {
  name: summary_none,
  justfile: "",
  args: ("--summary", "--quiet"),
  stdout: "\n\n\n",
}

#[test]
fn no_recipes() {
  Test::new()
    .arg("--summary")
    .stderr("Justfile contains no recipes.\n")
    .stdout("\n\n\n")
    .run();
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
    .run();
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
    .run();
}
