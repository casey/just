use super::*;

#[test]
fn search_directory_without_recipe() {
  Test::new()
    .justfile("foo:")
    .args(["--usage", "."])
    .stderr("error: `--usage` requires recipe\n")
    .failure();
}

#[test]
fn usage_recipe_in_search_directory() {
  Test::new()
    .justfile("foo bar:")
    .write("child/justfile", "foo:")
    .current_dir("child")
    .args(["--usage", "../foo"])
    .stdout(
      "
        Usage: just foo bar

        Arguments:
          bar
      ",
    )
    .success();
}

#[test]
fn usage() {
  Test::new()
    .justfile("mod bar")
    .write(
      "bar.just",
      "
set lists
[arg('a', short='a')]
[arg('b', pattern='123|789', help='hello')]
[arg('d', short='d', long='delightful')]
[arg('e', short='e', pattern='abc|xyz')]
[arg('f', long='f', pattern=['lucky', 'dog'])]
[arg('g', short='g', value='foo')]
foo a b c='abc' d e f='xyz' g='bar' *h:
",
    )
    .args(["--usage", "bar", "foo"])
    .unstable()
    .stdout(
      "
        Usage: just bar foo [OPTIONS] b [c] [h...]

        Arguments:
          b hello [pattern: '123|789']
          [c] [default: 'abc']
          [h...]

        Options:
          -a a
          -d, --delightful d
          -e e [pattern: 'abc|xyz']
              --f f [default: 'xyz'] [pattern: 'lucky' | 'dog']
          -g
      ",
    )
    .success();
}

#[test]
fn no_sections() {
  Test::new()
    .justfile("foo:")
    .args(["--usage", "foo"])
    .stdout("Usage: just foo\n")
    .success();
}

#[test]
fn arguments_only() {
  Test::new()
    .justfile("foo bar:")
    .args(["--usage", "foo"])
    .stdout(
      "
        Usage: just foo bar

        Arguments:
          bar
      ",
    )
    .success();
}

#[test]
fn options_only() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='b')]
        foo bar:
      ",
    )
    .args(["--usage", "foo"])
    .stdout(
      "
        Usage: just foo [OPTIONS]

        Options:
          -b bar
      ",
    )
    .success();
}

#[test]
fn arguments_and_options() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='b')]
        foo bar baz:
      ",
    )
    .args(["--usage", "foo"])
    .stdout(
      "
        Usage: just foo [OPTIONS] baz

        Arguments:
          baz

        Options:
          -b bar
      ",
    )
    .success();
}

#[test]
fn flags_have_no_value_placeholder() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', long, flag, help = 'a flag')]
        foo bar:
      ",
    )
    .args(["--usage", "foo"])
    .unstable()
    .stdout(
      "
        Usage: just foo [OPTIONS]

        Options:
              --bar a flag
      ",
    )
    .success();
}
