use super::*;

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
[arg('a', short='a')]
[arg('b', pattern='123|789', help='hello')]
[arg('d', short='d', long='delightful')]
[arg('e', short='e', pattern='abc|xyz')]
[arg('f', long='f', pattern='lucky')]
[arg('g', short='g', value='foo')]
foo a b c='abc' d e f='xyz' g='bar' *h:
",
    )
    .args(["--usage", "bar", "foo"])
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
              --f f [default: 'xyz'] [pattern: 'lucky']
          -g
      ",
    )
    .success();
}

#[test]
fn help_may_be_expression() {
  Test::new()
    .justfile(
      "
        prefix := 'hello '
        [arg('bar', help=prefix + 'world')]
        foo bar:
      ",
    )
    .args(["--usage", "foo"])
    .stdout(
      "
        Usage: just foo bar

        Arguments:
          bar hello world
      ",
    )
    .success();
}

#[test]
fn help_list_is_joined() {
  Test::new()
    .justfile(
      "
        set lists
        [arg('bar', help=['hello', 'world'])]
        foo bar:
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["--usage", "foo"])
    .stdout(
      "
        Usage: just foo bar

        Arguments:
          bar hello world
      ",
    )
    .success();
}

#[test]
fn help_empty_list_is_no_help() {
  Test::new()
    .justfile(
      "
        set lists
        [arg('bar', help=[])]
        foo bar:
      ",
    )
    .env("JUST_UNSTABLE", "1")
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
