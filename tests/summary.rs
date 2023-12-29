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
    .write("foo.just", "foo:")
    .justfile(
      "
        mod foo

        bar:
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("--summary")
    .stdout("bar foo::foo\n")
    .run();
}
